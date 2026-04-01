import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';

// AAA FIX: Standardized global reset. Removed redundant overflow rules.
const rootStyle = document.createElement('style');
rootStyle.innerHTML = `
  body, html {
    margin: 0;
    padding: 0;
    width: 100%;
    height: 100%;
    overflow: hidden;
    background-color: #111;
  }
  * {
    box-sizing: border-box;
  }
`;
document.head.appendChild(rootStyle);

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
    <App />
);