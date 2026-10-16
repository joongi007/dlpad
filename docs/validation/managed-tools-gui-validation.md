# 관리 도구 native GUI 격리 검증

2026-10-01. [작업 계약](../work-items/S04-managed-tools-probe.md).

`scripts/managed_tools_probe.py`는 기존 Xvfb/screenshot helper를 사용하며 visible desktop 옵션과 fallback이 없다. 임시 app-data/synthetic-media는 sibling이고 GUI process group만 종료/회수한다. 빌드는 수행하지 않는다. 기존 report prefix를 덮어쓰지 않는다.

install은 실제 제품 공식 고정 HTTPS pin을 UI로 설치하고 SQLite active/version/target, publication executable 크기/SHA256, manifest source와 digest를 대조한다. UseActive 클릭 후 downloads 입력을 Ctrl+A/C하여 격리 X clipboard의 정확 경로를 대조한다. 실제 영상 다운로드를 시작하지 않는다. cancel은 staging 존재를 관측한 뒤 중단을 누르고 staging 종료와 activation 부재를 확인한다. failure는 임시 소유 managed-root의 foreign fixture로 ownership 실패/원본 보존을 확인하고 오류 문구 screenshot은 별도 review한다.

AST parse 및 validate-only 인자8건 통과: install/cancel/failure 정상, NaN/181초 timeout·잘못된 mode·화면 밖 좌표·desktop 옵션 거절. 이 검증은 프로세스/GUI/네트워크를 실행하지 않았다. 아래 실제 binary 범위의 GUI 증거를 추가했다. 구현이나 인자 검증만으로 GUI gate 통과를 주장하지 않는다.

최종 binary 준비 후 실행 예:

```sh
DLPAD_TEST_FONT=<fixture-font> python3 scripts/managed_tools_probe.py target/release/dlpad /tmp/dlpad-managed-install.json
DLPAD_TEST_FONT=<fixture-font> python3 scripts/managed_tools_probe.py target/release/dlpad /tmp/dlpad-managed-cancel.json --mode cancel
DLPAD_TEST_FONT=<fixture-font> python3 scripts/managed_tools_probe.py target/release/dlpad /tmp/dlpad-managed-failure.json --mode failure
```

좌표는960x640/100% fixture 기준이며 actual screenshot로 확인한 뒤 필요시 명시 point 인자를 사용한다. 빠른 install 완료로 staging 관측 이전 activation이 발생한 경우 cancel 시험은 timeout/실패로 기록하며 성공으로 추정하지 않는다. other OS GUI·IME·screen reader를 검증하지 않는다.


## 실제 Linux/WSL Xvfb 결과

주 담당이 제공한 release binary SHA256 `f0aa2e335ce182f76290d71f41b9f7f8ffb965875c25b39b90d4965e93bdf842`에서 확인했다. 한글 fixture는 기존 Malgun.ttf를 읽기만 했고960x640/100%에서 install216,272/cancel350,272/UseActive150,444/toolinput250,350을 사용했다. 이 좌표는 같은 font/layout 범위의 기본값이며 다른 폰트는 screenshot을 확인하여 명시 point 인자를 지정한다. 빌드는 이 harness가 수행하지 않는다.

- [Install/UseActive JSON](managed-tools-gui/install.json), [설치 화면](managed-tools-gui/installed.png), [다운로드 경로 화면](managed-tools-gui/downloads.png): UI 설치 후 active `2026.08.19/x86_64-unknown-linux-gnu`, 공식40,446,224 byte·SHA256·manifest HTTPS source 검증, UseActive→다운로드 입력 clipboard의 전체 정확 경로 일치. 임시 경로는 이미 정리되어 screenshot의 문자열만 증거다.
- [Cancel JSON](managed-tools-gui/cancel.json), [중단 화면](managed-tools-gui/cancel.png): 실제 staging 시작 관측 후 중단 버튼→stage 없어짐/noactive/GUI reaped,4.417초. 실제 파일 버전 활성화 전 중단이다.
- [Failure JSON](managed-tools-gui/failure.json), [실패 화면](managed-tools-gui/failure.png): foreign synthetic 파일 보존/noactive 자동 대조. 이미지 직접 검토에서 “도구 작업 실패·취소. 이전 활성 버전은 유지합니다. 경로·네트워크·공급 파일 검증 상태를 확인하세요.” 표시를 확인했다. 자동 오류 문구 OCR을 구현하지 않아 JSON native_gui_verified는false를 유지한다.

개발 중 실패도 있었다: 첫 SHA 시험은 Python3.10에 file_digest가 없어 GUI 시작 전에 실패했고 streaming hash로 수정했다. Install v1/v3/v4/v5는 공식 설치 pin 검증까지 되었으나 좌표/clipboard fixture 실패를 성공으로 보고하지 않았다. font를 추가한 v2는 좌표가 달라 설치가 시작되지 않아 소유 harness에 SIGINT를 보내 finally로 GUI/Xvfb를 회수했다(종료130). 실제 screenshot로 좌표를 조정하고 owned-window keyboard focus와 정확한 uint8 KeyCode ABI를 적용한 v6에서 clipboard 대조가 통과했다. 초기 각 실패의 단일 원인을 모두 확정하지 않는다.

단일 고정 version만 시험했으므로 보관 버전 선택/제거 GUI와 기존 active가 있는 GUI update 실패/취소는 미검증이다. 기존 active 보존 backend 시험과 구분한다. 후속 자막 UI/core/schema 변경 이후 새 binary를 이 hash의 GUI 결과로 검증 완료라고 보고하지 않는다. 전체3OS/P0 출시 완료를 의미하지 않는다.
