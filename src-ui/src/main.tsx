import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';

// Reset browser default margins to ensure the FlexLayout grid perfectly hugs the OS window bounds.
const rootStyle = document.createElement('style');
rootStyle.innerHTML = `
  body, html {
    margin: 0;
    padding: 0;
    width: 100%;
    height: 100%;
    overflow: hidden;
    background-color: #1e1e1e;
  }
  * {
    box-sizing: border-box;
  }
`;
document.head.appendChild(rootStyle);

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
    // StrictMode intentionally disabled here. In WebGPU engine dev, double-mounting 
    // the WASM memory on boot causes fatal adapter locking.
    <App />
);