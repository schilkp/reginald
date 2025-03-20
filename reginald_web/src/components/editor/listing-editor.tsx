import { useRef, lazy, Suspense } from "react"
import type * as monaco from "monaco-editor"

// Import Monaco setup and editor together
const Editor = lazy(async () => {
    // We load both in parallel, but we only return the editor module
    const [, editorModule] = await Promise.all([
        import("../../lib/monaco-setup").then(module => {
            module.setupMonaco();
            return module;
        }),
        import("@monaco-editor/react")
    ]);
    return editorModule;
})

interface ListingEditorProps {
    value: string
    language: string
}

export const exampleYaml = `
---
name: MAX77654
defaults:
  layout_bitwidth: 8

enums:
  REG_EN:
    bitwidth: 3
    enum:
      FPS_SLOT_0:
        val: 0x0
        doc: FPS slot 0
      FPS_SLOT_1:
        val: 0x1
        doc: FPS slot 1
      FPS_SLOT_2:
        val: 0x2
        doc: FPS slot 2
      FPS_SLOT_3:
        val: 0x3
        doc: FPS slot 3
      DISABLED:
        val: 0x4
        doc: Off irrespective of FPS
      ENABLED:
        val: 0x6
        doc: On irrespective of FPS

  INT_MASK:
    bitwidth: 1
    enum:
      UNMASKED:
        val: 0
        doc: "Enabled/Unmasked"
      MASKED:
        val: 1
        doc: "Disabled/Masked"

registers:
  INT_GLBL0: !Register
    adr: 0x00
    reset_val: 0x00
    doc: Global Interrupt flag register 0.
    layout: !Layout
      DOD0_R:
        bits: 7
        access: [R]
        doc: LDO Dropout Detector Rising Interrupt
        accepts: !Bool
      DOD1_R:
        bits: 6
        access: [R]
        doc: LDO Dropout Detector Rising Interrupt
        accepts: !Bool
      TJAL2_R:
        bits: 5
        access: [R]
        doc: Thermal Alarm 2 Rising Interrupt
        accepts: !Bool
      TJAL1_R:
        bits: 4
        access: [R]
        doc: Thermal Alarm 1 Rising Interrupt
        accepts: !Bool
      nEN_R:
        bits: 3
        access: [R]
        doc: nEN Rising Interrupt
        accepts: !Bool
      nEN_F:
        bits: 2
        access: [R]
        doc: nEN Falling Interrupt
        accepts: !Bool
      GPI0_R:
        bits: 1
        access: [R]
        doc: GPI0 Rising Interrupt
        accepts: !Bool
      GPI0_F:
        bits: 0
        access: [R]
        doc: GPI0 Falling Interrupt
        accepts: !Bool

  INT_GLBL1: !Register
    adr: 0x04
    reset_val: 0x00
    doc: Global Interrupt flag register 1.
    layout: !Layout
      RESERVED:
        bits: 7
        accepts: !Fixed 0
      LDO1_F:
        bits: 6
        access: [R]
        doc: LDO1 Fault Interrupt
        accepts: !Bool
      LDO0_F:
        bits: 5
        doc: LDO0 Fault Interrupt
        access: [R]
        accepts: !Bool
      SBB_TO:
        bits: 4
        access: [R]
        doc: SBB Timeout
      GPI2_R:
        bits: 3
        access: [R]
        doc: GPI Rising Interrupt
        accepts: !Bool
      GPI2_F:
        bits: 2
        access: [R]
        doc: GPI Falling Interrupt
        accepts: !Bool
      GPI1_R:
        bits: 1
        access: [R]
        doc: GPI Rising Interrupt
        accepts: !Bool
      GPI1_F:
        bits: 0
        access: [R]
        doc: GPI Falling Interrupt
        accepts: !Bool`

export default function ListingEditor({ value, language }: ListingEditorProps) {
    const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null)

    const handleEditorDidMount = (editor: monaco.editor.IStandaloneCodeEditor) => {
        editorRef.current = editor
    }

    return (
        <div className="h-full w-full">
            <Suspense fallback={<div className="flex items-center justify-center h-full">Loading editor...</div>}>
                <Editor
                    height="100%"
                    defaultLanguage={language}
                    value={value}
                    onMount={handleEditorDidMount}
                    options={{
                        minimap: { enabled: false },
                        scrollBeyondLastLine: true,
                        fontSize: 12,
                        wordWrap: "on",
                        automaticLayout: true,
                    }}
                />
            </Suspense>
        </div>
    )
}



