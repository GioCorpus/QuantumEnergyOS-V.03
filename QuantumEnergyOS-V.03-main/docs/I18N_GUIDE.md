# Guía de Internacionalización (i18n) para QuantumEnergyOS

## Estructura de archivos

```
public/locales/
├── en/
│   ├── common.json     # Navegación, UI común, errores
│   ├── dashboard.json  # Mensajes del tablero
│   ├── quantum.json    # Mensajes de operaciones cuánticas
│   └── errors.json     # Mensajes de error
└── es/
    ├── common.json
    ├── dashboard.json
    ├── quantum.json
    └── errors.json
```

## Cómo agregar un nuevo idioma

### Paso 1: Crear el archivo de traducciones

Copiar `public/locales/en/common.json` a `public/locales/{lang-code}/common.json`

### Paso 2: Traducir todas las claves

Cada clave debe tener su equivalente traducido:

```json
{
  "dashboard": {
    "title": "Mi Título Traducido",
    "peakDemand": "Demanda Máxima"
  }
}
```

### Paso 3: Actualizar App.tsx

Agregar el nuevo idioma al selector:

```typescript
languages: [
  { code: 'es', name: 'Español', flag: '🇲🇽' },
  { code: 'en', name: 'English', flag: '🇺🇸' },
  { code: 'fr', name: 'Français', flag: '🇫🇷' },  // Nuevo
]
```

### Paso 4: Actualizar el archivo de configuración i18n

Editar `src/i18n.ts` para incluir el nuevo idioma:

```typescript
resources: {
  en: { ... },
  es: { ... },
  fr: {
    common: () => import('../locales/fr/common.json'),
    dashboard: () => import('../locales/fr/dashboard.json'),
  },
}
```

## Uso en componentes

### Hook useTranslation

```typescript
import { useTranslation } from 'react-i18next';

function MyComponent() {
  const { t } = useTranslation(['common', 'dashboard']);
  
  return (
    <h1>{t('dashboard.title')}</h1>
  );
}
```

### Placeholders

```typescript
// En JSON:
{ "hour": "Hora: {{hour}}" }

// En código:
t('hour', { hour: 14 })  // Resultado: "Hora: 14"
```

## Backend Python (Flask)

El archivo `i18n.py` proporciona funciones de traducción para el API:

```python
from i18n import translate, get_locale

# En endpoints:
@app.get("/api/status")
def status():
    locale = get_locale()
    return jsonify({
        "message": translate("health.healthy", locale),
        "locale": locale
    })
```

## Detección automática de idioma

1. Header `Accept-Language` del navegador
2. Cookie `qeos_language` guardada en preferencias
3. Valor por defecto: inglés (`en`)

## Pruebas

```bash
# Verificar que todas las traducciones existen
npm run lint:i18n

# Construir con verificación
npm run build
```

## Idiomas soportados actualmente

- 🇲🇽 **es-MX** - Español (México) - **Principal**
- 🇬🇧 **en** - English (US) - Secundario

## Siguientes idiomas planeados

- 🇫🇷 **fr** - Français
- 🇵🇹 **pt** - Português
- 🇩🇪 **de** - Deutsch
- 🇨🇳 **zh** - 中文