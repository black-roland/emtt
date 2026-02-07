; This Source Code Form is subject to the terms of the Mozilla Public
; License, v. 2.0. If a copy of the MPL was not distributed with this
; file, You can obtain one at https://mozilla.org/MPL/2.0/.

Unicode True
!include "MUI2.nsh"
!include "nsDialogs.nsh"
!include "LogicLib.nsh"

!define APPNAME "EMtT"
!ifndef VERSION
  !define VERSION "1.1.0"
!endif
!ifndef DIST_DIR
  !define DIST_DIR "dist"
!endif

Name "${APPNAME} ${VERSION}"
OutFile "${DIST_DIR}/emtt-${VERSION}-windows-amd64-setup.exe"
InstallDir "$LOCALAPPDATA\Programs\EMtT"
RequestExecutionLevel user

VIProductVersion "${VERSION}.0"
VIAddVersionKey "ProductName" "Easy Meshtastic to Telegram"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "ProductVersion" "${VERSION}"
VIAddVersionKey "FileDescription" "Установщик Easy Meshtastic to Telegram"
VIAddVersionKey "LegalCopyright" "MPL-2.0"

Var Dialog
Var TokenField
Var ChatIdField
Var DmCheck
Var ChannelField

Var token
Var chatid
Var dm
Var channel
Var params

!define MUI_ABORTWARNING
!define MUI_ICON "${NSISDIR}\Contrib\Graphics\Icons\modern-install.ico"
!define MUI_UNICON "${NSISDIR}\Contrib\Graphics\Icons\modern-uninstall.ico"

; Страницы установщика
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "LICENSE"
!insertmacro MUI_PAGE_DIRECTORY

; Страница настроек
Page custom ConfigPage ConfigPageLeave

!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

; Деинсталлятор
!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

!insertmacro MUI_LANGUAGE "Russian"

!define MUI_WELCOMEPAGE_TITLE "Добро пожаловать в установщик Easy Meshtastic to Telegram"
!define MUI_WELCOMEPAGE_TEXT "Этот установщик разместит EMtT в папке пользователя.\n\nПрава администратора не потребуются.\n\nНажмите «Далее» для продолжения."

Section "Main"
  SetOutPath "$INSTDIR"

  File "${DIST_DIR}/emtt.exe"
  File "LICENSE"

  WriteUninstaller "$INSTDIR\uninstall.exe"

  ; Ярлыки только для текущего пользователя
  SetShellVarContext current
  CreateDirectory "$SMPROGRAMS\EMtT"
  CreateShortcut "$SMPROGRAMS\EMtT\Easy Meshtastic to Telegram.lnk" "$INSTDIR\emtt.exe" "$params" "" 0 SW_SHOWNORMAL "" "Easy Meshtastic to Telegram"

  ; Запись об удалении — только для текущего пользователя (HKCU)
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\EMtT" "DisplayName" "${APPNAME}"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\EMtT" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\EMtT" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\EMtT" "Publisher" "man smart-home"

SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\emtt.exe"
  Delete "$INSTDIR\LICENSE"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"

  SetShellVarContext current
  Delete "$SMPROGRAMS\EMtT\Easy Meshtastic to Telegram.lnk"
  RMDir "$SMPROGRAMS\EMtT"

  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\EMtT"
SectionEnd

; Страница конфигурации
Function ConfigPage
  !insertmacro MUI_HEADER_TEXT "Настройка Easy Meshtastic to Telegram" "Укажите параметры для ярлыка в меню «Пуск»"

  nsDialogs::Create 1018
  Pop $Dialog

  ${If} $Dialog == error
    Abort
  ${EndIf}

  ; Более компактная компоновка — всё помещается без обрезки
  ${NSD_CreateLabel} 0u 5u 100% 10u "Токен Telegram-бота (обязательно):"
  Pop $0
  ${NSD_CreatePassword} 0u 18u 100% 12u ""
  Pop $TokenField

  ${NSD_CreateLabel} 0u 35u 100% 10u "ID чата Telegram (обязательно):"
  Pop $0
  ${NSD_CreateText} 0u 48u 100% 12u ""
  Pop $ChatIdField

  ${NSD_CreateLabel} 0u 65u 100% 10u "Пересылать личные сообщения из меш-сети:"
  Pop $0
  ${NSD_CreateCheckbox} 0u 78u 100% 12u "Включено (по умолчанию)"
  Pop $DmCheck
  ${NSD_SetState} $DmCheck ${BST_CHECKED}

  ${NSD_CreateLabel} 0u 95u 100% 10u "Пересылать сообщения из канала (опционально, «0» — основной канал):"
  Pop $0
  ${NSD_CreateText} 0u 108u 100% 12u ""
  Pop $ChannelField

  nsDialogs::Show
FunctionEnd

Function ConfigPageLeave
  ${NSD_GetText} $TokenField $token
  ${NSD_GetText} $ChatIdField $chatid
  ${NSD_GetState} $DmCheck $0
  ${If} $0 == ${BST_CHECKED}
    StrCpy $dm "true"
  ${Else}
    StrCpy $dm "false"
  ${EndIf}
  ${NSD_GetText} $ChannelField $channel

  ${If} $token == ""
    MessageBox MB_OK|MB_ICONEXCLAMATION "Токен бота Telegram обязателен!"
    Abort
  ${EndIf}
  ${If} $chatid == ""
    MessageBox MB_OK|MB_ICONEXCLAMATION "ID чата Telegram обязателен!"
    Abort
  ${EndIf}

  StrCpy $params 'syslog --bot-token "$token" --chat-id "$chatid" --dm $dm'
  ${If} $channel != ""
    StrCpy $params '$params --channel "$channel"'
  ${EndIf}
FunctionEnd
