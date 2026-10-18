# ADR-0014: 현재 사용자 Windows 앱 등록과 제거 진입

- 상태: Proposed, 구현·실제 Windows 인수 중
- 날짜: 2026-10-19
- 관련: R08, R11; [설치기 수명](0009-installer-launch-supervision.md), [업데이트 부모](0013-windows-update-bootstrap.md)

## 결정 후보

현재 사용자 HKCU Uninstall entry·Programs .lnk와 별도 LocalAppData의 private shell support를 연결한다. bundle 내부 Registered는 파일 검증 단계이며 OS 등록과 다르다. 원 bundle ID/canonicalpath/SID에서 installation ID를 구하고 다른 설치/외부 key·lnk를 덮어쓰지 않는다. Windows 목록의 초기 버전은 검증된 baseline bundle 버전이다. shortcut은 baseline 부모를 실행하므로 기존 signed cold 선택을 유지한다. 전체 ZIP7/manifest5·bundle strict namespace를 바꾸지 않고 installer PE를 별도 제거기로 재사용한다. 신규 crate는 없고 기존 windows-sys의 Registry/COM/Shell binding만 활성화한다. helper copy와 owner 자료의 디스크·실행 비용은 실제 검증 후 기록한다.

제거 진입은 원 제거기 복사본의 currentSID/nativeID/원 bytes를 확인하고 private temporary copy·명시 HANDLE/nonce/원 부모 종료 ACK로 인계한다. shell 문자열·Job breakaway·무소유 외부 프로세스·관리자 권한은 도입하지 않는다. temp 실행 파일의 자기 삭제는 이번 범위에 넣지 않고 고유 private 임시 산출물로 남긴다. 전역 temp 청소나 사용자 파일·프로필·도구·외부 install journal 삭제는 하지 않는다.

명시 제거는 원 bundle/journal 일치와 원 삭제 capability를 다시 획득한다. 실행 중인 앱을 임의 PID로 종료하지 않고 원 삭제 preflight가 거절해야 한다. shell 원 capability를 먼저 관측하되 bundle 부재가 확인된 뒤에만 registry/shortcut/support를 해제한다. 부분 실패/cleanup 미확인은 성공이 아니며 등록을 먼저 없애지 않는다. 같은 프로세스의 원 실패 capability 재시도와 cold 부분 제거 복구는 다르며 후자는 별도 인수 없이 완료를 주장하지 않는다.

기존0.1.2 설치는 새 설치기의 readonly 재검증 뒤 별도 Windows 등록 행동으로 연결한다. 재검증 자체는 등록 쓰기를 하지 않는다. 새 packaged 설치는 앱 본체 설치 뒤 OS 등록까지 시도하고 등록만 실패하면 본체를 보존하며 재시도 동작을 제공한다. 포터블 UI는 OS 등록을 자동 수행하지 않는다.

## 근거와 인수

[Microsoft registry API](https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regcreatekeyexw), [Shell Link](https://learn.microsoft.com/en-us/windows/win32/shell/links)의 명시 HANDLE/COM API를 사용한다. 실제 Windows 시험, 기존 updater 호환, clean registration/launch/uninstall/reinstall·충돌/변조/실행중 거절과 private GUI 결과를 작업 문서에 기록한 뒤 채택 여부를 갱신한다.

## 병렬 제거와 부분 실패

같은 설치를 검사하거나 제거하는 worker는 현재 사용자만 접근 가능한 설치별 Windows mutex를 유지한다. 이 mutex는 원 파일·레지스트리 삭제 권한을 대신하지 않으며, 원 capability가 모두 닫힌 뒤 해제한다. 다른 제거 창이 확인을 기다리는 동안 두 번째 창은 기존 창을 사용하거나 닫은 뒤 재시도하도록 안내한다.

삭제 권한을 일부만 얻은 상태에서 파일이 잠겨 있으면 이름·원 native ID·원 weak HANDLE을 모두 보존한다. 같은 실패 객체의 재시도는 남은 원 권한 획득부터 이어가며, 전체 권한 확인 전에 삭제를 시작하지 않는다. 새로운 경로 관측만으로 소유권을 다시 만들지 않는다.

Windows Shell에 표시할 DOS/UNC 경로와 내부 파일 검증용 경로는 구분한다. 표기 변환 시 원 파일 및 부모의 native ID를 대조하며, trailing dot/space 등 Win32에서 의미가 달라지는 경로는 거절한다. 긴 경로의 Shell 동작은 별도 실제 회귀로 검증한다.

등록된 설치의 제거는 Windows 설치 목록의 전용 제거기로 통일한다. 기존 설치기의 단발 제거 worker는 shell 정리 실패 권한을 유지할 수 없으므로 등록된 bundle의 삭제를 시작하지 않는다. 설치기는 등록 상태를 읽기만 하는 검사 후 ‘제거 방법’으로 Windows 설정 경로를 안내한다. 미등록 legacy bundle의 기존 명시 제거는 유지한다. 등록 완료 후에는 원 bundle의 새 검증 결과에 따라 앱 실행 버튼을 복구한다.
