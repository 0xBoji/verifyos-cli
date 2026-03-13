import * as fs from "node:fs";
import * as path from "node:path";
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

function bundledBinaryName(): string | undefined {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === "darwin" && arch === "arm64") {
    return path.join("bin", "darwin-arm64", "voc");
  }

  if (platform === "darwin" && arch === "x64") {
    return path.join("bin", "darwin-x64", "voc");
  }

  if (platform === "linux" && arch === "x64") {
    return path.join("bin", "linux-x64", "voc");
  }

  if (platform === "win32" && arch === "x64") {
    return path.join("bin", "win32-x64", "voc.exe");
  }

  return undefined;
}

function resolveBundledBinary(context: vscode.ExtensionContext): string | undefined {
  const relative = bundledBinaryName();
  if (!relative) {
    return undefined;
  }

  const absolute = context.asAbsolutePath(relative);
  return fs.existsSync(absolute) ? absolute : undefined;
}

function serverCommand(context: vscode.ExtensionContext): { command: string; args: string[]; source: string } {
  const config = vscode.workspace.getConfiguration("verifyOS");
  const profile = config.get<string>("profile", "basic");
  const useBundledBinary = config.get<boolean>("useBundledBinary", true);
  const bundledBinary = useBundledBinary ? resolveBundledBinary(context) : undefined;
  const command = bundledBinary ?? config.get<string>("path", "voc");
  return {
    command,
    args: ["lsp", "--profile", profile],
    source: bundledBinary ? "bundled" : "configured",
  };
}

async function startClient(context: vscode.ExtensionContext): Promise<void> {
  if (client) {
    return;
  }

  const server = serverCommand(context);
  const channel = output();
  channel.appendLine(`Starting verifyOS language server via ${server.source} binary: ${server.command}`);
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
      if (
        event.affectsConfiguration("verifyOS.path")
        || event.affectsConfiguration("verifyOS.profile")
        || event.affectsConfiguration("verifyOS.useBundledBinary")
      ) {
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
