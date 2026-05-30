export const TOOL_DECLARATIONS = [
  {
    name: "qiongli_literature_status",
    description: "Report configured literature providers and capability mode without exposing secrets.",
    inputSchema: {
      type: "object",
      additionalProperties: false,
      properties: {}
    }
  },
  {
    name: "qiongli_literature_search",
    description: "Search academic literature using configured OpenAlex and Semantic Scholar providers.",
    inputSchema: {
      type: "object",
      additionalProperties: true,
      properties: {
        query: {
          type: "string"
        },
        limit: {
          type: "number"
        }
      }
    }
  },
  {
    name: "qiongli_literature_export_evidence",
    description: "Export an auditable provider capability and search evidence snapshot.",
    inputSchema: {
      type: "object",
      additionalProperties: true,
      properties: {}
    }
  }
];

export function listTools() {
  return TOOL_DECLARATIONS;
}

export async function handleToolCall(name) {
  const tool = TOOL_DECLARATIONS.find((candidate) => candidate.name === name);
  if (!tool) {
    throw new Error(`Unknown tool: ${name}`);
  }

  return {
    content: [
      {
        type: "text",
        text: JSON.stringify({
          tool: name,
          status: "not_implemented",
          message: "Qiongli Literature Provider MCPB skeleton is installed; provider tools will be implemented later."
        })
      }
    ]
  };
}

export async function startStdioServer() {
  const [{ Server }, { StdioServerTransport }, { CallToolRequestSchema, ListToolsRequestSchema }] =
    await Promise.all([
      import("@modelcontextprotocol/sdk/server/index.js"),
      import("@modelcontextprotocol/sdk/server/stdio.js"),
      import("@modelcontextprotocol/sdk/types.js")
    ]);

  const server = new Server(
    {
      name: "qiongli-literature-provider",
      version: "0.1.0"
    },
    {
      capabilities: {
        tools: {}
      }
    }
  );

  server.setRequestHandler(ListToolsRequestSchema, async () => ({
    tools: listTools()
  }));

  server.setRequestHandler(CallToolRequestSchema, async (request) => {
    return handleToolCall(request.params.name);
  });

  await server.connect(new StdioServerTransport());
}

function isDirectRun() {
  return import.meta.url === `file://${process.argv[1]}`;
}

if (isDirectRun()) {
  await startStdioServer();
}
