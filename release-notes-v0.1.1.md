## Citrus AutoFish v0.1.1 — prerelease для Horizon Launcher 0.2.69

Исправлена упаковка исходников под CLI Horizon Launcher 0.2.69: нативный mod bundle находится в `mixins/citrus_autofish` и содержит `horizon-mixin.toml`. В v0.1.0 был использован новый корневой формат `horizon-mod.toml`, который Launcher 0.2.69 не распознаёт.

### Возможности
- `/autofish` включает и выключает режим.
- После серверного события поклёвки ставится в очередь обычный клик Horizon для вытягивания.
- Через 4 игровых тика после вытягивания ставится клик для следующего заброса.
- Ввод проходит через штатный `click_mapping("key.use")`; мод не конструирует игровые пакеты вручную.

### Установка на Windows / PowerShell
Из распакованной папки релиза запустите:

```powershell
& "C:\Users\ADMIN\AppData\Local\Programs\Horizon Launcher\horizon-launcher.exe" native-build .\mixins\citrus_autofish --trust-native-code
```

Если лаунчер установлен иначе, замените путь к `horizon-launcher.exe`. Важно передать именно папку `mixins\citrus_autofish`, не корень архива. После сборки зайдите в одиночный мир и введите `/autofish`.

Для Bash/Linux/macOS из корня распакованного проекта:

```sh
horizon-launcher native-build ./mixins/citrus_autofish --trust-native-code
```

### Совместимость и ограничения
Проект закреплён за Minecraft 26.3 и коммитом Horizon в `horizon-mixin.toml`. Предполагаемый сценарий — одиночный мир. На удалённых vanilla-серверах серверный сигнал поклёвки может быть недоступен.

Это экспериментальный исходный релиз без собранного бинарника или `.hnmod`. Сборка и игровой тест на реальном лаунчере не выполнялись; исходники и manifest проверены статически. Native-мод работает с правами пользователя и не изолирован песочницей — проверяйте код перед разрешением `--trust-native-code`.
