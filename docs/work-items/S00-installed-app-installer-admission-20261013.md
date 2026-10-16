# S00: 기존 앱 설치 상태와 설치 버튼

- 상태: in-progress
- 분류: bug
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: installer_batch — installer_main.rs, installer_catalog.rs 후보; root — 통합/검증
- 선행 작업: v20 사용자 UI 검수

## 사용자 결과

이미 존재하는 target에서는 설치를 활성화하지 않으며 기존 파일/경로를 삭제하거나 덮어쓰지 않는다. 기존 설치에 대한 검증·실행의 실제 지원 상태를 설명하고 worker에서 다시 검사한다. UI thread에서 fs/native I/O 금지.

## 계약과 범위

Rust native UI와 기존 소유권/취소/CloseWait를 유지한다. 각 후보는 별도 namespace만 수정하고 제품 소스 채택은 root가 조정한다. 공유 catalog는 root만 수정한다.

## 인수 조건

이미 존재하는 target에서는 설치를 활성화하지 않으며 기존 파일/경로를 삭제하거나 덮어쓰지 않는다. 기존 설치에 대한 검증·실행의 실제 지원 상태를 설명하고 worker에서 다시 검사한다. UI thread에서 fs/native I/O 금지.

## 검증과 handoff

구현·실제 시험 및 시각 검수 결과를 기록한다. 미실행은 통과로 보고하지 않는다.

## 2026-10-14 통합 checkpoint

presence admission3/0, 기본 위치 정책8/0 격리 검토 후 v21 개발 소스에 통합했다. 기존 폴더/파일/journal과 Unknown/Pending에서는 Install을 거절하며 Verify 안내와 현재 default ID를 제공한다. 실제 기존 설치기 private UI helper는 새 whole/payload 인수 뒤 실행할 준비이며 아직 실행 완료가 아니다.
