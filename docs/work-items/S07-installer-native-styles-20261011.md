# S07: 설치 화면 native 글꼴과 버튼 스타일

- 상태: in-progress
- 분류: polish
- 우선순위: P0
- 요구사항: R01, R08
- 담당/수정 범위: root crates/app/build.rs, installer_main.rs, assets/dlpad-installer.*, scripts/generate_installer_resources.py, scripts/test_installer_resources.py; timeline_precision /tmp 후보
- 선행 작업: S07 packaged installer

## 사용자 결과

설치 화면은 OS message font와 native visual styles를 사용한다.

## 계약과 범위

기존 아이콘 bytes를 유지하고 설치 binary에만 CommonControls v6 manifest와 초기화를 연결한다. 다른 앱 binary resource는 그대로다. 추가 crate/권한 요구 없음. 새 source resource 보관은 약 573 KiB이며 실행파일 증가·native 동작은 실제 검사한다. GNU resource parsing은 target OS 실행 검증과 다르다.

## 인수 조건

RES/COFF 원 icon 보존 회귀, 실제 MSVC linking/실행 및 private PNG 비교, 기존 설치/재검증/folder 동작 통과.

## 검증과 handoff

root 통합 후 검증 중. DPI/IME/실제 사용자 OS 테마 전체 인수를 주장하지 않는다.

2026-10-11 통합: corrected whole에서 INITCOMMONCONTROLSEX 지역 변수 controls가 기존 controls 함수를 가리는 E0618을 발견했다. common_controls로 이름을 수정하고 새 unshadowed whole에서 재검증 중이다. 실패 근거는 보존했다.

최신 full-installed-local-av-v6 실제 private GUI install/reverify 자연0·Job0 통과. root가 installed5와 journal5 SHA를 원 명세/receipt에 독립 대조하고 initial/verified PNG를 직접 확인했다. unsigned/cleanOS/사용자 OS 최종 인수는 별도이며 전체P0 완료가 아니다.

최신binary의fullpayload설치·재검증PNG와원5파일/journal5SHA는rootPASS_SCOPED. 동일v6바이너리의상위폴더선택/취소/modalownerclose는새3회귀로재확인예정이며982당시근거를소급하지않는다.
