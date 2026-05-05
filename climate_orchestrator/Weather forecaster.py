import requests
from datetime import datetime

print("🔴 QuantumEnergyOS - Orquestador Climatológico")
print("="*50)

lat, lon = 32.63, -115.45
url = f"https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}&current=temperature_2m&hourly=temperature_2m&timezone=America/Tijuana"

data = requests.get(url).json()
temp = data  print(f"🌡️  Temperatura actual en Mexicali: {temp}°C")
print(f"🕒 Hora: {datetime.now().strftime('%H:%M')}")

print("\n📢 ESTADO DE LA RED:")

if temp >= 45:
    print("🔴 ALERTA CRÍTICA - 45°C o más")
    print("   Demanda extrema esperada. Preparar cortes rotativos.")
elif temp >= 42:
    print("🟠 ALERTA ALTA - 42°C a 44°C")
    print("   Riesgo alto de sobrecarga. Activar medidas preventivas.")
elif temp >= 38:
    print("🟡 ALERTA MEDIA - 38°C a 41°C")
    print("   Monitoreo constante recomendado.")
else:
    print("🟢 Normal")
