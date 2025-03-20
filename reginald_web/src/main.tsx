import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'

//import * as monaco from 'monaco-editor';
//import { loader } from '@monaco-editor/react';
//
//loader.config({ monaco });

import * as wasm from "reginald_wasm";

wasm.hello_world();

createRoot(document.getElementById('root')!).render(
    <StrictMode>
        <App />
    </StrictMode>,
)
