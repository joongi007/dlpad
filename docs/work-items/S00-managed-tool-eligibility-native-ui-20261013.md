# S00: 관리 도구 자동 선택의 실제 GUI 검수

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R02, R08, R11
- 담당/수정 범위: windows_auto — 새 전용 private C# probe 후보; root — 실행·이미지·수명 검토
- 선행 작업: 제품 whole20 성공; 원 private desktop probe의 소유권/취소/자연 종료 계약

## 사용자 결과

관리 도구 자동 선택의 실제 GUI 검수를 기존 Windows 네이티브 흐름에 연결하고 실제 실패/표시 동작을 검증한다.

## 계약과 범위

후보는 `.local-tools/session-recovery-20261013/` 새 namespace에 보존한다. 같은 파일을 동시에 수정하지 않으며 root가 공유 계약·제품 source freeze·통합 검증을 조정한다.

## 인수 조건

Download의 도구 및 인증 disclosure를 열어 실제 100/200% Light/Dark 자동 선택·없음 설명·수동 옵션과 스크롤을 캡처한다. 사용자 Default desktop/global input/clipboard/임의 도구 다운로드 금지.

## 검증과 handoff

whole20 앱63은 직접 검토했으나 이 도구 선택 설명은 기존 캡처 viewport 밖이다. 별도 probe를 준비 중이며 임의 root-visible fallback 또는 전체 GUI PASS로 대체하지 않는다.

## 2026-10-13 표시 검수 보류

Dark100 비공개 데스크톱 실행은 자연 종료 및 disabled Auto/enabled Manual 속성을 수집했다. root가 PNG를 직접 확인하니 tool0 안내문이 캡처 viewport 아래에 있고 dropdown이 footer 위로 겹쳐 나타났다. 수집 시점 geometry와 캡처 시점의 일치 여부를 windows_auto가 별도 v2에서 조사한다. 원래 v1 및 interop compile 실패 증거는 보존하며 전체 설명 배치 통과로 보고하지 않는다.

## 재검수 결과

v2 Dark100은 root가 7 PNG 전부 직접 검수하고 원본 native/source/pin을 확인했다. 안내문·닫힌 combo·열린 선택 popup은 본문 영역 안에 들어오고 actual disabled Auto/enabled Manual 및 popup 닫힘 identity/value ACK가 확인됐다. 원 v1 판정은 여백 34px 누락에 따른 시험 guard 오류이며 해당 설명 배치 PASS로 사용하지 않는다. v2 Light200 첫 호출 outer143은 native compile 이전 중단으로 원인 미확인; 별도 retry 실행은 32회 joint-scroll guard 실패로 미통과했다. 해당 실패 PNG에서 combo와 설명은 보이지만 popup 여유까지 정착되지 않아 popup 자체의 배치는 아직 미검증이다. 실패 cleanup은 소유 Job만 정리했으며 사용자 desktop은 unchanged였다. 최신 installer GDI ignored 시험 3개는 별도 `windows-msvc-current20-private-gdi-20261013`에서 3 PASS/0 FAIL이다.
