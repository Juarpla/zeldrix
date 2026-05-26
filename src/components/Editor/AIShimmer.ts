import { Mark, mergeAttributes } from "@tiptap/react";

export const AIShimmer = Mark.create({
  name: "aiShimmer",

  addOptions() {
    return {
      HTMLAttributes: {
        class: "ai-shimmer-active",
      },
    };
  },

  parseHTML() {
    return [
      {
        tag: "span.ai-shimmer-active",
      },
    ];
  },

  renderHTML({ HTMLAttributes }) {
    return [
      "span",
      mergeAttributes(this.options.HTMLAttributes, HTMLAttributes),
      0,
    ];
  },
});
