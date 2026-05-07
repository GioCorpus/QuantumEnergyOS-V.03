import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import '../i18n';

const root = ReactDOM.createRoot(
  document.getElementById('root') as HTMLElement
);

root.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);

// Add i18n initialization
if (typeof window !== 'undefined') {
  const savedLang = localStorage.getItem('qeos-language');
  if (savedLang) {
    document.documentElement.lang = savedLang;
  }
}