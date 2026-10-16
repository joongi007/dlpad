# S01: 최신 파일 조회의 bounded UTF16 경로 buffer 재사용

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root crates/engine/src/library/windows_scan.rs·windows_scan/fresh.rs·회귀/비용/CONTEXT/ADR0002; metrics_recovery /tmp 읽기검토
- 선행 작업: 실제 quietworker 비용은 파일조사/기록이 지배, 임시index 최적화뒤 평균1.248%로미달

## 사용자 결과

파일별 최신 metadata 조회에서 같은 경로 버퍼를 다시 할당하지 않도록 줄인다.

## 계약과 범위

Api가65534byte(32767 UTF16 units) 용량의scratch Vec 하나를 소유하고 조회마다 clear한다. 기존prefix/UNC/verbatim/NUL/32766경계와 full128ID/volume/size/mtime·no-reparse fallback 검사를 유지한다. query/DLL은 owned Api수명 안에 있고 shared mutable/thread 권한을 추가하지 않는다. Cargo/lock/의존성/schema 변경 없음. 64KiB 미만 추가 최대용량은 walker 하나에 한정하고 실패 시 drop한다.

## 인수 조건

정상/한글/UNC/verbatim·NUL·길이경계·실패 후 재사용 회귀, Windows native scanner/fallback/alias/취소 시험 유지. 같은 quietworker 원fixture 전후비용과 최신전체/GUI 범위를 실제검증한다. scan주기/metadata freshness 완화없음.

## 검증과 handoff

구현 전 읽기검토와 이전baseline핀을 먼저 고정한다. 값검사·루트종류·native class unsupported fallback은 그대로 유지한다.

### 네이티브 회귀

첫 fmt는 함수와 UNC fixture의 줄바꿈 2건으로 실패했고 원 로그를 보존했다. 교정 후 fmt·기존 profile fixture 포함 Clippy·Windows library27회귀가 통과했다. 버퍼 재사용/경계 2개와 기존 상대경로·NUL·oversize/원 HANDLE alias 시험도 포함한다. 독립 검토에서 두 수정 파일이 native green snapshot과 일치함을 확인했다. 같은 quietworker 비용을 새 경로에서 재측정 중이며 P0/최신 GUI 통과로 확대하지 않는다.

### 2026-10-06 실제 통합 결과

Windows library27회귀 및 최신 전체4검증이 통과했다. source541 현재CPU bb60a5d5e3a5aec90940061077fbf3155c723ebb7689b27e8af98f5af5964927/7,895,040B를 root·독립 검토했다. quietworker 평균1.22221%는 미달이고 lazyindex1.24799%와 한 쌍 차이로 강한 CPU 개선을 주장하지 않는다. 약64KiB 경로 scratch를 walker마다 재사용한다. 최신 GUI 예산은 미검증이다. [전체 증거](../validation/evidence/windows-library-observation-cost-workspace-20261006-continue/result.json).
