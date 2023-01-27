import * as pulumi from "@pulumi/pulumi";
import * as aws from "@pulumi/aws-native";
import * as awsc from "@pulumi/aws";
import { local } from "@pulumi/command";
import { execSync } from "child_process";

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


const api = new aws.apigatewayv2.Api(`food-${stack}`, {
    protocolType: "HTTP",
});

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
});

const lambdaRoleAttachment = new awsc.iam.RolePolicyAttachment("lambdaRoleAttachment", {
    role: lambdaRole,
    policyArn: awsc.iam.ManagedPolicy.AWSLambdaBasicExecutionRole,
});

const buildLambda = new local.Command("buildlambda", {
    create: `./build.sh 2>&1`,
    update: `./build.sh 2>&1`,
    dir: '../backend',
    environment: {
    },
    triggers: [execSync('shasum ../backend/src/* | shasum ')]
})

export const buildOutput = buildLambda.stdout;
export const buildError = buildLambda.stderr;

const lambda = new awsc.lambda.Function("lambdaFunction", {
    code: new pulumi.asset.FileArchive("../backend/target/lambda/backend"),
    runtime: "provided.al2",
    architectures: ["arm64"],
    role: lambdaRole.arn,
    handler: "bootstrap",
}, {dependsOn: buildLambda});

const apigw = new awsc.apigatewayv2.Api("httpApiGateway", {
    protocolType: "HTTP",
});

export const apiUrl = apigw.apiEndpoint;

const lambdaPermission = new awsc.lambda.Permission("lambdaPermission", {
    action: "lambda:InvokeFunction",
    principal: "apigateway.amazonaws.com",
    function: lambda,
    sourceArn: pulumi.interpolate`${apigw.executionArn}/*/*`,
}, { dependsOn: [apigw, lambda] });

const integration = new awsc.apigatewayv2.Integration("lambdaIntegration", {
    apiId: apigw.id,
    integrationType: "AWS_PROXY",
    integrationUri: lambda.arn,
    integrationMethod: "POST",
    payloadFormatVersion: "2.0",
    passthroughBehavior: "WHEN_NO_MATCH",
});

const route = new awsc.apigatewayv2.Route("apiRoute", {
    apiId: apigw.id,
    routeKey: "$default",
    target: pulumi.interpolate`integrations/${integration.id}`,
});

const stage = new awsc.apigatewayv2.Stage("apiStage", {
    apiId: apigw.id,
    name: stack,
    routeSettings: [
        {
            routeKey: route.routeKey,
            throttlingBurstLimit: 5000,
            throttlingRateLimit: 10000,
        },
    ],
    autoDeploy: true,
}, { dependsOn: [route] });

export const endpoint = pulumi.interpolate`${apigw.apiEndpoint}/${stage.name}`;

