export type LlmComplete = (input: {
  prompt: string;
  source: string;
  signature: string;
  neighbors: string;
  instruction: string;
}) => Promise<string>;

export function mockLlmComplete(): LlmComplete {
  return async ({ prompt, source, signature, instruction }) => {
    const text = `${prompt}\n${instruction}`.toLowerCase();
    if (text.includes("retry") || text.includes("重试")) {
      return applyRetry(source, signature);
    }
    if (instruction.startsWith("ghost:")) {
      const name = instruction.slice("ghost:".length).trim() || "generated";
      return `${source}\n\ndef ${name}() -> str:\n    return "generated"\n`;
    }
    if (instruction.startsWith("insert-call:")) {
      const callee = instruction.slice("insert-call:".length).trim();
      return insertCall(source, signature, callee);
    }
    return source;
  };
}

export async function httpLlmComplete(opts: {
  apiKey: string;
  baseUrl: string;
  model: string;
}): Promise<LlmComplete> {
  return async (input) => {
    const url = `${opts.baseUrl.replace(/\/$/, "")}/chat/completions`;
    const res = await fetch(url, {
      method: "POST",
      headers: {
        authorization: `Bearer ${opts.apiKey}`,
        "content-type": "application/json",
      },
      body: JSON.stringify({
        model: opts.model,
        messages: [
          {
            role: "system",
            content:
              "You edit Python source. Return ONLY the full new file contents, no markdown.",
          },
          {
            role: "user",
            content: `Signature: ${input.signature}\nNeighbors: ${input.neighbors}\nInstruction: ${input.instruction}\nPrompt: ${input.prompt}\n\nFILE:\n${input.source}`,
          },
        ],
      }),
    });
    if (!res.ok) {
      throw new Error(`LLM HTTP ${res.status}`);
    }
    const json = (await res.json()) as {
      choices?: { message?: { content?: string } }[];
    };
    const content = json.choices?.[0]?.message?.content;
    if (!content) {
      throw new Error("LLM returned empty content");
    }
    return content.replace(/^```(?:python)?\n?|\n?```$/g, "");
  };
}

function applyRetry(source: string, signature: string): string {
  const name = functionName(signature);
  return replaceFunction(source, name, (body) => {
    const indented = body
      .split("\n")
      .map((line) => (line ? `    ${line}` : line))
      .join("\n");
    return `def ${name}${paramsOf(signature)}:\n    last = None\n    for _attempt in range(3):\n        try:\n${indented}\n        except Exception as exc:\n            last = exc\n    raise last\n`;
  });
}

function insertCall(source: string, signature: string, callee: string): string {
  const name = functionName(signature);
  const callName = callee.split(".").pop() ?? callee;
  return replaceFunction(source, name, (full) => {
    const trimmed = full.replace(/\s*$/, "");
    return `${trimmed}\n    ${callName}()\n`;
  });
}

export function functionName(signature: string): string {
  const match = /def\s+([A-Za-z_][\w]*)/.exec(signature);
  return match?.[1] ?? "fn";
}

function paramsOf(signature: string): string {
  const idx = signature.indexOf("(");
  return idx >= 0 ? signature.slice(idx) : "()";
}

export function replaceFunction(
  source: string,
  name: string,
  next: (currentDef: string) => string,
): string {
  const re = new RegExp(`^def ${name}\\(.*$`, "m");
  const match = re.exec(source);
  if (!match || match.index === undefined) {
    return `${source}\n\n${next(`def ${name}():\n    pass\n`)}`;
  }
  const start = match.index;
  const lines = source.slice(start).split("\n");
  const defLines = [lines[0]];
  for (let i = 1; i < lines.length; i++) {
    if (/^(def |class |@)/.test(lines[i]) && !lines[i].startsWith(" ")) {
      break;
    }
    defLines.push(lines[i]);
  }
  const current = defLines.join("\n");
  const end = start + current.length;
  return source.slice(0, start) + next(current) + source.slice(end);
}
