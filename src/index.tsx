import {createRoot} from "react-dom/client";
import {StrictMode} from "react";
import {createBrowserRouter} from "react-router-dom";
import { App } from "./App";

const container = document.getElementById('root');
const root = createRoot(container!);
root.render(
  <StrictMode>
    <App />
  </StrictMode>
)