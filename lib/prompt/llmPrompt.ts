
export type LLMPrompt = {
  model: string;
  max_tokens: number;
  system: string;
  tools: any[];
  messages: any[];
};

export const getLlmPrompt = () => {
  return {
    model: "claude-3-sonnet-20240229",
    max_tokens: 400,
    system:
      "Before answering the question, please think about it step-by-step within <thinking></thinking> tags. Then, provide your final answer within <answer></answer> tags. Skip the introduction and start from <thinking> tag.",
    tools: [
      {
        name: "get_weather",
        description:
          "Get the current weather in a given location. Weather is defined base on city name and state or country.",
        input_schema: {
          type: "object",
          properties: {
            location: {
              type: "string",
              description:
                "The city and state, <example>San Francisco, CA</example> <example>Berlin, Germany</example>",
            },
          },
          required: ["location"],
        },
      },
      {
        name: "get_restaurants",
        description:
          "Get list of recommended restaurants in the given city. It provides information if the given facility offers outdoor seatings. Restaurants are grouped by reviews from guests",
        input_schema: {
          type: "object",
          properties: {
            location: {
              type: "string",
              description:
                "The city and state, <example>San Francisco, CA</example> <example>Berlin, Germany</example>",
            },
          },
          required: ["location"],
        },
      },
    ],
    "messages.$": "$.messages",
  };
};
