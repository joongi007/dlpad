# S03: Windows 앱 내 도구 설치와 설치 화면의 선택 공급

- 상태: implemented
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R07, R08, R11
- 담당/수정 범위: root 공유 module/catalog, installer_tools.rs, engine/tools/{direct_download,archive_download}.rs, 통합·검증·문서; metrics_recovery app/{managed,managed_candidates,accessible_tools}.rs; soak_recovery_audit app/{installer_main,installer_catalog}.rs. 하위 담당 완료 후 root가 수정·통합했다. 같은 파일의 동시 수정 없음.
- 선행 작업: 기존 관리 도구 archive 다운로드·검증·설치·선택 엔진

## 사용자 결과

Windows x64 앱의 도구 화면에서 yt-dlp·Deno·FFmpeg/ffprobe를 설치할 수 있다. 설치 화면에도 기본 해제된 세 선택 항목을 추가했다. 앱 시작 시 기존 활성 도구를 빈 실행 파일 경로에 복원한다. 직접 지정한 외부 경로와 그동안의 사용자 수정은 보존한다.

## 계약과 범위

[ADR 0008](../adr/0008-windows-optional-tool-install.md), [도구 계약](../INTEGRATIONS.md)을 따른다. 고정 HTTPS source·길이·SHA·notice·component manifest를 기존 owned worker에 연결했다. archive 도구는 설치 후 보관 상태이며 앱에서 활성화·사용해야 한다. 원 direct yt-dlp 설치는 버전 검증과 활성화를 함께 수행한다.

설치 프로그램은 앱 prepare 성공 뒤 선택 도구를 순차 설치한다. 원 취소와 절대 300초 deadline을 공유하며 각 도구는 최대 120초 및 남은 전체 기한 중 작은 한도를 적용한다. 실패하면 다음 도구를 시작하지 않는다. 앱 설치와 도구 공급 결과는 별도이며 기존 활성 도구를 보존한다. 도구 root가 source/bundle/journal과 겹치면 앱 mutation 전에 거절한다. 이 사전 containment 검사는 외부 동시 경로 교체의 원자적 방지 증명이 아니다.

개발용 --tool-data는 절대 경로 시험 프로필이다. 실제 GUI 시험은 모두 소유 TEMP에 격리했다. 사용자 Documents·영상·계정·쿠키를 사용하지 않았다. Rust native 유지, Cargo/schema/lock 변경 없음. 정상 Git 저장소가 없어 기존 .git을 보존했다.

## 인수 조건

세 도구의 실제 설치·버전 검증·활성화·Use 경로 적용과 재실행 복원을 검증한다. 설치 화면의 기본 해제·선택 공급·앱 결과 독립성을 확인한다. preCancel/expired/foreign owner/경로 겹침/사용자 수정/닫기 중 적용 거절 등 실패 회귀를 실행한다. 물리 입력·미검증 OS·실제 사이트·정식 배포로 확대하지 않는다.

## 검증과 handoff

실행: `python3 scripts/windows_msvc_workspace_probe.py --configuration docs/validation/evidence/windows-msvc-folder-ui-close-safe-workspace-retry-20261007/configuration.json --output docs/validation/evidence/windows-msvc-ui-polish-tool-final-20261008 --reuse-owned-target` (실행 시 절대 경로 사용). Windows fmt, Clippy, locked workspace test, release build 모두 PASS, **934 passed / 0 failed / 16 ignored**, 610개 입력 동일, 원 4개 process 자연 종료·Job0·private/input 보호. [root 검토](../validation/evidence/windows-msvc-ui-polish-tool-final-20261008/root-review.json).

`windows_current_native_ui_collect.py --kind tools`: [실제 앱 검토](../validation/evidence/windows-msvc-ui-polish-tools-restore-wait-20261008/root-review.json). 공식 HTTPS 세 도구 설치, archive 두 도구 보관→활성화→Use의 정확한 경로 적용, 재실행 세 경로 복원, 두 앱 자연 종료. root와 독립 담당이 source/pins/receipt/SQLite active 3행·integrity·읽기 전후 SHA와 6개 component/notice SHA·정확한 namespace를 검토했다. 최종 whole와 차이는 installer 두 파일뿐이며 CPU bytes/SHA가 같다는 결합 근거를 기록했다.

`windows_msvc_native_installer_optional_collect.py`: [최종 선택 설치 검토](../validation/evidence/windows-msvc-ui-polish-installer-final-20261008/root-review.json). 기본 세 항목 해제를 확인한 뒤 Deno만 선택해 실제 설치했다. 앱 재검증·별도 공급 결과·inactive 도구 DB·자연 종료를 확인했다. root와 독립 담당이 현재 source/25pins/원 로그/앱·저널 보존/Deno와 notice SHA를 확인하고 PNG 3장을 직접 봤다. 안내 줄 붙음도 캡처에 따라 수정·재검증했다.

실패 자료는 보존했다. 첫 Clippy는 test helper의 미등록 의존성으로 실패해 새 의존성 없이 소유 TEMP helper로 고쳤다. disclosure 회귀는 처음 잘못된 부모 ID를 사용해 실패했다. 실제 Windows adapter에 ExpandCollapse provider가 없어 Toggle/Click을 함께 제공했다. 재실행 관측기는 복구 중 비활성 탐색을 즉시 호출해 실패한 뒤 기존 제한 시간 안의 활성 대기로 고쳤다. optional collector의 이전 fixture prefix 불일치도 원 GUI 성공과 수집 실패를 구분해 교정했다.

CPU 9,404,416B, installer 4,290,560B, CLI 974,848B. installer는 기존 HTTPS/TLS·archive 코드를 처음 링크해 이전 1,103,360B보다 3,187,200B 증가했다. 새 의존성/feature/lock 변경이나 작업 없는 앱의 자동 다운로드는 없다.

남은 후속: 설치 중 실제 GUI 취소·CloseACK, 실제 사이트 다운로드, clean OS, 서명·법적 공지 완결·정식 설치/배포. 사용자 최종 UI 인수와 전체 P0는 미완료다. updater와 CI/CD는 기존 후속 계획을 유지한다.
