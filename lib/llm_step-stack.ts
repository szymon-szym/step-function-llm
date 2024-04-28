import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";
import * as sfn from "aws-cdk-lib/aws-stepfunctions";
import * as events from "aws-cdk-lib/aws-events";
import * as tasks from "aws-cdk-lib/aws-stepfunctions-tasks";
import * as secrets from "aws-cdk-lib/aws-secretsmanager";
import * as lambda from "aws-cdk-lib/aws-lambda";
import { getLlmPrompt } from "./prompt/llmPrompt";

export class LlmStepStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const toolsLambda = new lambda.Function(this, "llm_tools_lambda", {
      runtime: lambda.Runtime.PROVIDED_AL2,
      code: lambda.Code.fromAsset(
        "lib/functions/llm_tools/target/lambda/llm_tools/"
      ),
      handler: "not.required",
      memorySize: 256,
      timeout: cdk.Duration.seconds(30),
    });

    const llmAPIKey = new secrets.Secret(this, "LlmApiKey", {});

    const llmDestinationConnection = new events.Connection(
      this,
      "LlmDestinationConnection",
      {
        authorization: events.Authorization.apiKey(
          "x-api-key",
          llmAPIKey.secretValue
        ),
        description: "LLM Destination Connection",
      }
    );

    const callLlmTask = new tasks.HttpInvoke(this, "Call LLM", {
      apiRoot: "https://api.anthropic.com",
      apiEndpoint: sfn.TaskInput.fromText("/v1/messages"),
      body: sfn.TaskInput.fromObject(getLlmPrompt()),
      connection: llmDestinationConnection,
      headers: sfn.TaskInput.fromObject({
        "Content-Type": "application/json",
        "anthropic-version": "2023-06-01",
        "anthropic-beta": "tools-2024-04-04",
      }),
      method: sfn.TaskInput.fromText("POST"),
      resultSelector: {
        "role.$": "$.ResponseBody.role",
        "content.$": "$.ResponseBody.content",
        "stop_reason.$": "$.ResponseBody.stop_reason",
      },
      resultPath: "$.taskResult",
    });

    
    const passAnswer = new sfn.Pass(this, "Answer");
    
    const callToolsTask = new tasks.LambdaInvoke(this, "Call tools", {
      lambdaFunction: toolsLambda,
      resultSelector: {
        "messages.$": "$.Payload",
      },
    }).next(callLlmTask);
    
    const choiceIfUseTool = new sfn.Choice(this, "Choice if use tool");

    choiceIfUseTool.when(
      sfn.Condition.stringEquals("$.taskResult.stop_reason", "tool_use"),
      callToolsTask
    );

    choiceIfUseTool.otherwise(passAnswer);

    const stateMachine = new sfn.StateMachine(this, "LlmStateMachine", {
      definition: callLlmTask.next(choiceIfUseTool),
    });
  }
}
