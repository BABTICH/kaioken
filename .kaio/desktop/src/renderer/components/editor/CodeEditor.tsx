// src/renderer/components/editor/CodeEditor.tsx
import { useEffect, useRef, useState, useCallback } from "react";
import { EditorState, Compartment, type Extension } from "@codemirror/state";
import { EditorView, lineNumbers, highlightActiveLine, highlightActiveLineGutter, drawSelection, dropCursor, keymap } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { closeBrackets, closeBracketsKeymap, autocompletion, completionKeymap } from "@codemirror/autocomplete";
import { searchKeymap, highlightSelectionMatches } from "@codemirror/search";
import { bracketMatching, indentOnInput, foldGutter, foldKeymap } from "@codemirror/language";
import { useStore } from "@nanostores/react";
import { resolvedTheme } from "@renderer/stores/theme";
import { kaiokenDarkTheme, kaiokenLightTheme } from "@renderer/lib/editor/kaiokenTheme";
import { getLanguageForFile } from "@renderer/lib/editor/getLanguage";

interface CodeEditorProps {
  content: string;
  filename: string;
  readOnly?: boolean;
  onChange?: (value: string) => void;
  onCursorChange?: (line: number, col: number) => void;
}

const themeCompartment = new Compartment();
const languageCompartment = new Compartment();

export function CodeEditor({ content, filename, readOnly = true, onChange, onCursorChange }: CodeEditorProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const viewRef = useRef<EditorView | null>(null);
  const theme = useStore(resolvedTheme);

  // Initialize editor
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const baseExtensions: Extension[] = [
      lineNumbers(),
      highlightActiveLineGutter(),
      foldGutter(),
      drawSelection(),
      dropCursor(),
      EditorState.allowMultipleSelections.of(true),
      indentOnInput(),
      bracketMatching(),
      closeBrackets(),
      autocompletion(),
      highlightActiveLine(),
      highlightSelectionMatches(),
      history(),
      keymap.of([
        ...closeBracketsKeymap,
        ...defaultKeymap,
        ...searchKeymap,
        ...historyKeymap,
        ...foldKeymap,
        ...completionKeymap
      ]),
      EditorView.lineWrapping,
      themeCompartment.of(theme === "dark" ? kaiokenDarkTheme : kaiokenLightTheme),
      languageCompartment.of([]),
      EditorView.editable.of(!readOnly),
      EditorState.readOnly.of(readOnly),
      EditorView.updateListener.of((update) => {
        if (update.docChanged && onChange) {
          onChange(update.state.doc.toString());
        }
        if (update.selectionSet && onCursorChange) {
          const cursor = update.state.selection.main.head;
          const line = update.state.doc.lineAt(cursor);
          onCursorChange(line.number, cursor - line.from + 1);
        }
      })
    ];

    const state = EditorState.create({
      doc: content,
      extensions: baseExtensions
    });

    const view = new EditorView({
      state,
      parent: container
    });

    viewRef.current = view;

    // Load language asynchronously
    void getLanguageForFile(filename).then((lang) => {
      if (lang && viewRef.current) {
        viewRef.current.dispatch({
          effects: languageCompartment.reconfigure(lang)
        });
      }
    });

    return () => {
      view.destroy();
      viewRef.current = null;
    };
    // Only run on mount; content/filename updates handled via separate effects
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [filename]);

  // Update theme when it changes
  useEffect(() => {
    if (!viewRef.current) return;
    viewRef.current.dispatch({
      effects: themeCompartment.reconfigure(
        theme === "dark" ? kaiokenDarkTheme : kaiokenLightTheme
      )
    });
  }, [theme]);

  // Update content when it changes externally
  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    const currentContent = view.state.doc.toString();
    if (currentContent !== content) {
      view.dispatch({
        changes: { from: 0, to: currentContent.length, insert: content }
      });
    }
  }, [content]);

  return (
    <div
      ref={containerRef}
      className="code-editor-container"
      style={{
        height: "100%",
        width: "100%",
        overflow: "hidden",
        display: "flex",
        flexDirection: "column"
      }}
    />
  );
}
