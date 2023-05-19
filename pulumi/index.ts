import * as pulumi from "@pulumi/pulumi";
import * as aws from "@pulumi/aws-native";
import * as awsc from "@pulumi/aws";
import { local } from "@pulumi/command";
import { execSync } from "child_process";
import * as fs from "fs";

let stack = pulumi.getStack();

const table = new aws.dynamodb.Table(`food-${stack}`, {
    attributeDefinitions: [
        { attributeName: "PK", attributeType: "S" },
        { attributeName: "SK", attributeType: "S" },
    ],
    keySchema: [{ AttributeName: "PK", KeyType: "HASH" }, { AttributeName: "SK", KeyType: "RANGE" }],
    billingMode: "PAY_PER_REQUEST"
});
export const tableName = table.tableName;

const lambdaRole = new awsc.iam.Role("lambdaRole", {
    assumeRolePolicy: {
        Version: "2012-10-17",
        Statement: [
            {
                Action: "sts:AssumeRole",
                Principal: {
                    Service: "lambda.amazonaws.com",
                },
                Effect: "Allow",
                Sid: "",
            },
        ],
    },
    managedPolicyArns: ["arn:aws:iam::aws:policy/AmazonDynamoDBFullAccess"]
});

const lambdaRoleAttachment = new awsc.iam.RolePolicyAttachment("lambdaRoleAttachment", {
    role: lambdaRole,
    policyArn: awsc.iam.ManagedPolicy.AWSLambdaBasicExecutionRole,
});

const execSyncShaCommand = `find ../lambdas -type f ! -path "*target*" -print0 | xargs -0 shasum | shasum`;

const buildLambda = new local.Command("buildlambda", {
    create: `./build.sh 2>&1`,
    update: `./build.sh 2>&1`,
    dir: '../lambdas',
    environment: {
    },
    triggers: [execSync(execSyncShaCommand)]
})

export const buildOutput = buildLambda.stdout;
export const buildError = buildLambda.stderr;

const apigw = new awsc.apigatewayv2.Api(`food-${stack}`, {
    protocolType: "HTTP",
});

const lambdas = [];

function foodLambda(name: string, routeKey: string) {
    const lambda = new awsc.lambda.Function(name, {
        code: new pulumi.asset.FileArchive(`../lambdas/target/lambda/` + name),
        runtime: "provided.al2",
        architectures: ["arm64"],
        role: lambdaRole.arn,
        handler: "bootstrap",
        environment: {
            variables: {
                TABLE_NAME: pulumi.interpolate`${tableName}`,
            }
        },
    }, {dependsOn: buildLambda});

    const lambdaPermission = new awsc.lambda.Permission(`${name}-permission`, {
        action: "lambda:InvokeFunction",
        principal: "apigateway.amazonaws.com",
        function: lambda,
        sourceArn: pulumi.interpolate`${apigw.executionArn}/*/*`,
    }, { dependsOn: [apigw, lambda] });

    const integration = new awsc.apigatewayv2.Integration(`${name}-integration`, {
        apiId: apigw.id,
        integrationType: "AWS_PROXY",
        integrationUri: lambda.arn,
        integrationMethod: "POST",
        payloadFormatVersion: "2.0",
        passthroughBehavior: "WHEN_NO_MATCH",
    });    

    const route = new awsc.apigatewayv2.Route(`${name}-route`, {
        apiId: apigw.id,
        routeKey: routeKey,
        target: pulumi.interpolate`integrations/${integration.id}`,
    });
}

const lambdaAddMeal = foodLambda("add-meal", "POST /meal");
const lambdaAddMealToDay = foodLambda("add-meal-to-day", "PATCH /day/{day}");
const lambdaGetMealById = foodLambda("get-meal-by-id", "GET /meal/{meal_id}");
const lambdaSearchMeal = foodLambda("search-meal", "GET /meal");


export const apiUrl = apigw.apiEndpoint;

const stage = new awsc.apigatewayv2.Stage("apiStage", {
    apiId: apigw.id,
    name: stack,
    defaultRouteSettings:
    {
        throttlingBurstLimit: 5000,
        throttlingRateLimit: 10000,
    },
    autoDeploy: true,
}, { dependsOn: [] });

export const endpoint = pulumi.interpolate`${apigw.apiEndpoint}/${stage.name}`;

