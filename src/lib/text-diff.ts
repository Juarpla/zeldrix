export type DiffSegmentType = "unchanged" | "added" | "removed";

export interface DiffSegment {
  type: DiffSegmentType;
  text: string;
}

export function htmlToPlainText(html: string): string {
  if (typeof document !== "undefined") {
    const element = document.createElement("div");
    element.innerHTML = html;
    return element.textContent ?? "";
  }

  return html.replace(/<[^>]*>/g, "");
}

export function diffText(before: string, after: string): DiffSegment[] {
  const beforeTokens = tokenize(before);
  const afterTokens = tokenize(after);
  const table = buildLcsTable(beforeTokens, afterTokens);
  const segments: DiffSegment[] = [];

  let beforeIndex = beforeTokens.length;
  let afterIndex = afterTokens.length;

  while (beforeIndex > 0 || afterIndex > 0) {
    if (
      beforeIndex > 0 &&
      afterIndex > 0 &&
      beforeTokens[beforeIndex - 1] === afterTokens[afterIndex - 1]
    ) {
      segments.unshift({
        type: "unchanged",
        text: beforeTokens[beforeIndex - 1],
      });
      beforeIndex -= 1;
      afterIndex -= 1;
    } else if (
      afterIndex > 0 &&
      (beforeIndex === 0 ||
        table[beforeIndex][afterIndex - 1] >= table[beforeIndex - 1][afterIndex])
    ) {
      segments.unshift({ type: "added", text: afterTokens[afterIndex - 1] });
      afterIndex -= 1;
    } else {
      segments.unshift({ type: "removed", text: beforeTokens[beforeIndex - 1] });
      beforeIndex -= 1;
    }
  }

  return compactSegments(segments);
}

function tokenize(value: string): string[] {
  return value.split(/(\s+)/).filter((token) => token.length > 0);
}

function buildLcsTable(left: string[], right: string[]): number[][] {
  const table = Array.from({ length: left.length + 1 }, () =>
    Array.from({ length: right.length + 1 }, () => 0)
  );

  for (let leftIndex = 1; leftIndex <= left.length; leftIndex += 1) {
    for (let rightIndex = 1; rightIndex <= right.length; rightIndex += 1) {
      if (left[leftIndex - 1] === right[rightIndex - 1]) {
        table[leftIndex][rightIndex] = table[leftIndex - 1][rightIndex - 1] + 1;
      } else {
        table[leftIndex][rightIndex] = Math.max(
          table[leftIndex - 1][rightIndex],
          table[leftIndex][rightIndex - 1]
        );
      }
    }
  }

  return table;
}

function compactSegments(segments: DiffSegment[]): DiffSegment[] {
  return segments.reduce<DiffSegment[]>((acc, segment) => {
    const previous = acc.at(-1);
    if (previous && previous.type === segment.type) {
      previous.text += segment.text;
    } else {
      acc.push({ ...segment });
    }
    return acc;
  }, []);
}
