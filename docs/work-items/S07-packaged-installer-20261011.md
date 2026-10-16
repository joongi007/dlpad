# S07: 앱 파일을 포함한 Windows 개발 설치 묶음

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R08, R10, R11
- 담당/수정 범위: root 통합; installer_main.rs, installer_catalog.rs, 개발 payload generator/test, 검증·설치 안내
- 선행 작업: 현재 MSVC whole 검증 및 원 binary/notice 패키지 proof

## 사용자 결과

설치 파일 옆에 포함된 payload가 있으면 설치 명세와 원본을 내부에서 고정하고 숨긴다. 사용자는 새 설치 폴더를 입력하거나 상위 폴더를 선택한다. 설치 기록 위치는 같은 상위 폴더의 별도 새 경로로 자동 결정한다. 기존 명시 네 경로 개발 실행도 유지한다. 대상 자동 생성·기존 설치 교체·등록·서명·자동 updater는 이 slice의 완료로 주장하지 않는다.

## 계약과 범위

payload 탐색은 입력 편의이며 trust나 파일 삭제 권한이 아니다. 기존 worker의 bounded BundleManifest/SHA/native identity 검증과 새 root 생성·취소·deadline·CloseWait를 그대로 사용한다. 개발 ZIP의 CPU bytes를 payload/dlpad.exe로 명시 매핑하고 원 SHA를 결합한다. 프로젝트 LICENSE와 원 notice bytes를 그대로 포함한다. 전체 법적 공지 누락은 승격하지 않는다. 런타임 의존성/Cargo.lock 추가 없음.

## 인수 조건

missing payload fallback, 명세 directory/symlink 거절, 명세 손상 설치 거절, explicit argv 유지, source readonly/immutable request, 실제 격리 Windows 새 설치·재검증·취소·충돌 보존을 확인한다. 원 데이터를 삭제하거나 사용자 Documents를 시험에 쓰지 않는다.

## 검증과 handoff

2026-10-11 후보 구현을 통합했다. 아직 current whole/실제 packaged GUI 인수·최종 설치 ZIP은 준비 중이다. 서명·clean OS 및 정식 설치/upgrade는 미검증이다.

2026-10-11 설치 위치 중심 UI와 native 폴더 browser 후보를 통합했다. 읽기 전용 journal 표시를 위조해도 요청은 대상에서 다시 유도한다. private collector도 신규 UI에 맞추었으며 compile/actual GUI 결과는 다음 검증에서 기록한다.

새 UI actual 검증: Windows whole fmt/Clippy/test/release982passed/0failed/17ignored, source610 현재map 및 원 자연exit0/Job0/privateclosed를 root검토했다. 새 packaged native 설치/재검증에서 internalfields hidden·target-only·derivedreadonlyjournal·위조source/journal표시무시와실제installedCPU SHA 및 journal5SHA를 root 재계산했다. root initialPNG직접검토. 폴더 dialog선택/취소/ownerclose·fullpayload·최종ZIP·physical/cleanOS는 별도 미완료이며 시스템글꼴/빈결과영역 UI개선도 진행한다. 근거: [whole](../validation/evidence/windows-msvc-simple-installer-quality-whole-20261011/root-review.json), [actual packaged](../validation/evidence/windows-msvc-simple-packaged-installer-native-20261011/root-review.json).
