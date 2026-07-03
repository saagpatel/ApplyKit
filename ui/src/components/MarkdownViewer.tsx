import type { ReactElement, ReactNode } from "react";

interface Props {
  markdown: string;
}

function renderInline(text: string): ReactNode[] {
  const nodes: ReactNode[] = [];
  const pattern = /(\*\*[^*]+\*\*|`[^`]+`)/g;
  let lastIndex = 0;
  let match: RegExpExecArray | null;

  while ((match = pattern.exec(text)) !== null) {
    if (match.index > lastIndex) {
      nodes.push(text.slice(lastIndex, match.index));
    }

    const token = match[0];
    const key = `${match.index}-${token}`;
    if (token.startsWith("**")) {
      nodes.push(<strong key={key}>{token.slice(2, -2)}</strong>);
    } else {
      nodes.push(<code key={key}>{token.slice(1, -1)}</code>);
    }
    lastIndex = match.index + token.length;
  }

  if (lastIndex < text.length) {
    nodes.push(text.slice(lastIndex));
  }

  return nodes;
}

function renderParagraph(line: string, key: string) {
  return <p key={key}>{renderInline(line)}</p>;
}

export function MarkdownViewer({ markdown }: Props) {
  const lines = markdown.replace(/\r\n/g, "\n").replace(/\r/g, "\n").split("\n");
  const nodes: ReactElement[] = [];
  let bulletBuffer: string[] = [];
  let orderedBuffer: string[] = [];
  let quoteBuffer: string[] = [];
  let codeBuffer: string[] = [];
  let inCodeBlock = false;

  const flushBullets = () => {
    if (bulletBuffer.length === 0) {
      return;
    }
    const keyBase = `ul-${nodes.length}`;
    nodes.push(
      <ul key={keyBase}>
        {bulletBuffer.map((entry, idx) => (
          <li key={`${keyBase}-${idx}`}>{renderInline(entry)}</li>
        ))}
      </ul>
    );
    bulletBuffer = [];
  };

  const flushOrdered = () => {
    if (orderedBuffer.length === 0) {
      return;
    }
    const keyBase = `ol-${nodes.length}`;
    nodes.push(
      <ol key={keyBase}>
        {orderedBuffer.map((entry, idx) => (
          <li key={`${keyBase}-${idx}`}>{renderInline(entry)}</li>
        ))}
      </ol>
    );
    orderedBuffer = [];
  };

  const flushQuote = () => {
    if (quoteBuffer.length === 0) {
      return;
    }
    nodes.push(
      <blockquote key={`quote-${nodes.length}`}>
        {quoteBuffer.map((entry, idx) => (
          <p key={`quote-line-${idx}`}>{renderInline(entry)}</p>
        ))}
      </blockquote>
    );
    quoteBuffer = [];
  };

  const flushLooseBlocks = () => {
    flushBullets();
    flushOrdered();
    flushQuote();
  };

  lines.forEach((raw, idx) => {
    const line = raw.trimEnd();

    if (inCodeBlock) {
      if (line.startsWith("```")) {
        nodes.push(
          <pre key={`code-${idx}`} className="markdown-code">
            <code>{codeBuffer.join("\n")}</code>
          </pre>
        );
        codeBuffer = [];
        inCodeBlock = false;
        return;
      }
      codeBuffer.push(raw);
      return;
    }

    if (!line.trim()) {
      flushLooseBlocks();
      return;
    }

    if (line.trimStart().startsWith("<!--")) {
      flushLooseBlocks();
      return;
    }

    if (line.startsWith("```")) {
      flushLooseBlocks();
      inCodeBlock = true;
      codeBuffer = [];
      return;
    }

    if (line.startsWith("- ")) {
      flushOrdered();
      flushQuote();
      bulletBuffer.push(line.slice(2).trim());
      return;
    }

    const orderedMatch = /^(\d+)\.\s+(.+)$/.exec(line);
    if (orderedMatch) {
      flushBullets();
      flushQuote();
      orderedBuffer.push(orderedMatch[2].trim());
      return;
    }

    if (line.startsWith("> ")) {
      flushBullets();
      flushOrdered();
      quoteBuffer.push(line.slice(2).trim());
      return;
    }

    flushLooseBlocks();
    if (line.startsWith("### ")) {
      nodes.push(<h4 key={`h4-${idx}`}>{renderInline(line.slice(4))}</h4>);
      return;
    }
    if (line.startsWith("## ")) {
      nodes.push(<h3 key={`h3-${idx}`}>{renderInline(line.slice(3))}</h3>);
      return;
    }
    if (line.startsWith("# ")) {
      nodes.push(<h2 key={`h2-${idx}`}>{renderInline(line.slice(2))}</h2>);
      return;
    }
    if (line === "---") {
      nodes.push(<hr key={`hr-${idx}`} />);
      return;
    }
    nodes.push(renderParagraph(line, `p-${idx}`));
  });

  if (inCodeBlock) {
    nodes.push(
      <pre key="code-open" className="markdown-code">
        <code>{codeBuffer.join("\n")}</code>
      </pre>
    );
  }

  flushLooseBlocks();

  return <div className="markdown-viewer">{nodes}</div>;
}
