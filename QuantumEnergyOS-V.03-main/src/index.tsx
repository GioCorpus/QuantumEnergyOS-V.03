import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './i18n';

const root = ReactDOM.createRoot(
  document.getElementById('root') as HTMLElement
);

root.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);

// Set initial language on HTML tag
if (typeof window !== 'undefined') {
  const savedLang = localStorage.getItem('qeos-language');
  if (savedLang) {
    document.documentElement.lang = savedLang;
  }
}