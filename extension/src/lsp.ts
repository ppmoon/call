import * as vscode from "vscode";

/** Best-effort LSP call-hierarchy edges. Failures are ignored; tree-sitter remains the source of truth. */
export async function lspCallees(file: string, line: number): Promise<string[]> {
  try {
    const uri = vscode.Uri.file(file);
    const pos = new vscode.Position(Math.max(0, line - 1), 0);
    const prepared = await vscode.commands.executeCommand<vscode.CallHierarchyItem[]>(
      "vscode.prepareCallHierarchy",
      uri,
      pos,
    );
    const item = prepared?.[0];
    if (!item) {
      return [];
    }
    const outgoing = await vscode.commands.executeCommand<vscode.CallHierarchyOutgoingCall[]>(
      "vscode.provideOutgoingCalls",
      item,
    );
    return (outgoing ?? []).map((call) => call.to.name);
  } catch {
    return [];
  }
}
