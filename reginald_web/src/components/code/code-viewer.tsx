import { useRef } from "react"
import type * as monaco from "monaco-editor"
import { Editor } from "@monaco-editor/react"

interface CodeOutputProps {
    value: string
    language: string
}

export default function CodeOutput({ value, language }: CodeOutputProps) {
    const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null)

    const handleEditorDidMount = (editor: monaco.editor.IStandaloneCodeEditor) => {
        editorRef.current = editor
    }

    return (
        <div className="h-full w-full">
            <Editor
                height="100%"
                defaultLanguage={language}
                value={value}
                onMount={handleEditorDidMount}
                options={{
                    readOnly: true,
                    minimap: { enabled: false },
                    scrollBeyondLastLine: true,
                    fontSize: 12,
                    wordWrap: "on",
                    automaticLayout: true,
                }}
            />
        </div>
    )
}


