# S11: 새 설치의 기본 위치 제안

- 상태: in-progress
- 분류: UX
- 우선순위: P0
- 요구사항: R02, R08, R11
- 담당/수정 범위: windows_auto — engine/location.rs·신규installer_defaults.rs; installer_batch — installer_main.rs wiring(독점); root — 인터페이스·통합·문서/native검증
- 선행 작업: 사용자모든기본값요구/설치UI전체개선

## 사용자 결과

패키지설치첫화면에유효한새설치위치를추천하고사용자는바꿀수있다. 기존자료를덮어쓰지않고조회실패에는선택행동을유지한다.

## 계약과 범위

기존directories/BaseDirs를사용해 localdata를조회한다. engine::location::default_install_target_hint와installer_defaults::recommended_target은OptionPathBuf다. 창생성전readonly 조회만;Programs존재/정상localdir이면Programs/dlpad,없으면LocalData/dlpad-app. 기존target에는16개이하suffix중absentleaf만추천한다. 권한/lookup/UNC/abnormalUTF16/absolute/reparse/file/path경계실패는None이다. 실제creation/namespace/ownership/execution권한과별개이며기존worker검증을유지한다. packagedsource발견+target미입력때만적용하고명시CLI/이미입력/기존data/save위치를덮어쓰지않는다. 새dependency/Cargo/schema변경없음.

## 인수 조건

합성temp fixture로Programs/누락fallback/충돌자료보존/suffixexhaustion/오류/한글/긴값/reparse를검증한다. 현재문서/실제Windowsinstallerfirstframe에서기본값을확인하고explicituseroverride/설치보존을검증한다. userDocuments/실영상/기존데이터를시험에쓰지않는다.

## 검증과 handoff

병렬실제구현을시작했다. Native/Cargo실행은아직없으며root가sourcefreeze뒤실제전체/GUI/설치검증을수행한다. UI전수작업은별도work-item과함께관리한다.

구현: engine/location readonly helper와 app/installer_defaults wrapper, installer startup wiring을 통합했다. 실제 metadata/suffix/file/error 보존 회귀와 explicit/partial 입력 우선 회귀가 새 Windows 전체 테스트에 포함됐으며 전체 terminal/실제 가상 설치 첫화면과 사용자 override 검증은 대기 중이다. 정상 조회는 소수 metadata 호출, 최악은 조상 두경로각256+Programs1+후보16으로529호출이며 새crate/lock변경이 없다.
