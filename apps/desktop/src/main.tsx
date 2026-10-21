import React from "react";
import ReactDOM from "react-dom/client";
import { App } from "./App";
import { registerEditorListeners } from "./editorListeners";
import "./styles.css";

registerEditorListeners();

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
