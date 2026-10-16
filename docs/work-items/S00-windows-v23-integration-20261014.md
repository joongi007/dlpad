# S00: Windows v23 목록 개선·성능 진단 통합

- 상태: done / v23 scoped 통합 검증
- 분류: integration / validation
- 우선순위: P0
- 요구사항: R01, R03, R08, R10, R11
- 담당/수정 범위: Root — 공유 인터페이스·Live 세 파일·전체 검사·설치본/문서; advanced_units — 분리 목록 후보; windows_auto — 분리 진단 후보; installer_batch — 분리 native/package helper
- 선행 작업: 목록49시험, 진단4/관측15/strict cpu 및 fixture42/strict fixture, 각 독립 Root 검토

## 사용자 결과

파일 목록의 선택 정보는 고정32px 한 줄로 표시하고 첫 선택 뒤 같은 위치의 재클릭 대상을 유지한다. 성능 원인을 좁히는 계측은 기존 진단 feature의 명시 소유 fixture에서만 작동하며 일반 설치본에는 컴파일하지 않는다.

## 계약과 범위

crates/app/src/accessible_library.rs, accessible_metrics.rs, 새 metrics_worker_phase_probe.rs를 Root가 통합한다. Cargo/workspace/lockfile/schema/catalog/기본 관측 주기 변경은 없다. root-v23-source-adoption-v1에 원본 backup·before/after SHA와 source669→670을 기록했다. 정상 Git 조회exit128을 보존하며 기존.git을 재구성하지 않았다. 코드/도구/앱은 Rust 네이티브, 시험은 Windows private desktop와 새 합성 파일만 사용한다. 기존 v22 ZIP·원 성공/실패 기록을 삭제하거나 새후보 검증으로 사용하지 않는다.

## 인수 조건

현재 소스에 대해 fmt, all-target strict clippy, workspace test, release 실제 Windows 성공과 원로그·해시·lifetime 독립 검토. 새 selected/unselected 목록을800/960·앱100/150/200%에서 실제 private 화면으로 확인하고 exact32px정보/행위치/RuntimeID/파일보존을 대조한다. 실제 worker 단계 report는 별도 cached owned feature target에서 전체4검사·고정binary를 사용하고 private naturalexit 뒤만 읽는다. 새 설치/portable ZIP은 새whole/payload/원notice/CRC inventory로 검토한다. 실제물리IME/OS DPI/reader/사용자UI/clean OS/서명·공개배포 인수는 별도다.

## 검증과 handoff

실제 원명령과 결과는 각 evidence/작업 기록에 연결한다. 현재 Live 세 파일 채택 완료, source670, 새전체Windows 검사 진행 중이다. Root의 frozen pins.files 형태 가정 setup 실패와 Label/value 시험조회 실패를 보존했고 실제49목록시험은 수정한 시험조회계약으로 통과했다. 새 설치본·private 새화면·worker report는 아직 미검증이다. 이전strict18비용2구간FAIL을 해결된 것으로 표현하지 않는다.

### 완료 범위

v23 Windows 전체 네 명령은1536/0/42·43harness/source670이며 Root 독립 대조를 완료했다. 새 ZIP 두 개와 원notice/CRC 검토도 완료했다. private 목록6장면/24PNG는 UIA선택·고정32px·원행좌표·파일보존 범위로 통과했고 원픽셀은 글자 가운데 정렬/좁은창 이름폭 문제를 드러냈다. 이를 후속 v24에서 수정한다. v23 실제 설치 시험은 v24 후보로 대체하며 미실행으로 남긴다. 별도 진단feature 전체1570/0/42와 실제worker17tick 보고서를 독립 대조했다. 한 tick의 외부wall평균은 약0.77ms이며 빈 합성목록/dispatch 범위로, 기존strict비용 실패 원인 해결 또는 최신v24 비용 증거가 아니다. [v24 후속](S00-windows-v24-integration-20261014.md).
