import * as vscode from "vscode";
import * as lc from "vscode-languageclient/node";

let client: lc.LanguageClient | undefined;
let outputChannel: vscode.OutputChannel | undefined;

function output(): vscode.OutputChannel {
  if (!outputChannel) {
    outputChannel = vscode.window.createOutputChannel("verifyOS");
  }
  return outputChannel;
}

function serverCommand(): { command: string; args: string[] } {
  const config = vscode.workspace.getConfiguration("verifyOS");
  const command = config.get<string>("path", "voc");
  const profile = config.get<string>("profile", "basic");
  return {
    command,
    args: ["lsp", "--profile", profile],
  };
}

async function startClient(context: vscode.ExtensionContext): Promise<void> {
  if (client) {
    return;
  }

  const server = serverCommand();
  const channel = output();
  const clientOptions: lc.LanguageClientOptions = {
    documentSelector: [
      { scheme: "file", pattern: "**/Info.plist" },
      { scheme: "file", pattern: "**/*.plist" },
      { scheme: "file", pattern: "**/*.xcprivacy" },
    ],
    outputChannel: channel,
  };

  client = new lc.LanguageClient(
    "verifyOS",
    "verifyOS",
    {
      command: server.command,
      args: server.args,
    },
    clientOptions,
  );

  client.onDidChangeState(({ newState }) => {
    if (newState === lc.State.Stopped) {
      channel.appendLine("verifyOS language server stopped.");
    }
  });

  try {
    await client.start();
    context.subscriptions.push(channel);
  } catch (error) {
    client = undefined;
    channel.appendLine(String(error));
    const selection = "Install verifyOS-cli";
    const picked = await vscode.window.showErrorMessage(
      "verifyOS could not start `voc lsp`. Make sure `voc` is installed and available on PATH, or set verifyOS.path.",
      selection,
    );
    if (picked === selection) {
      void vscode.env.openExternal(vscode.Uri.parse("https://crates.io/crates/verifyos-cli"));
    }
  }
}

async function restartClient(context: vscode.ExtensionContext): Promise<void> {
  if (client) {
    await client.stop();
    client = undefined;
  }
  await startClient(context);
}

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  context.subscriptions.push(
    vscode.commands.registerCommand("verifyOS.restartLanguageServer", async () => {
      await restartClient(context);
    }),
    vscode.commands.registerCommand("verifyOS.showOutput", () => {
      output().show(true);
    }),
    vscode.workspace.onDidChangeConfiguration(async (event) => {
      if (event.affectsConfiguration("verifyOS.path") || event.affectsConfiguration("verifyOS.profile")) {
        await restartClient(context);
      }
    }),
  );

  await startClient(context);
}

export async function deactivate(): Promise<void> {
  if (client) {
    await client.stop();
    client = undefined;
  }
}
