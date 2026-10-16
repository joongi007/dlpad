# S01: 폴더 변경 알림 재연결

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: 주 담당; crates/engine/src/watch.rs, crates/app/src/main.rs, scripts/native_probe.py, 관련 문서
- 선행 작업: S01 native watcher와 transaction reconciliation

## 사용자 결과

폴더가 사라지거나 알림 backend가 실패해도 제한된 간격으로 재연결하고 놓친 변경을 다시 스캔한다.

## 계약과 범위

watch 하나, 최신 상태/dirty flag만 유지. 2초 재시도, root mtime 변화와 30초 reconciliation hint로 조용히 놓친 native event 보완. root symlink 거절 및 Unix dev/ino/다른 OS creation time 확인으로 같은 경로의 폴더 교체 감지. 재연결만으로 DB 목록을 삭제하지 않는다. channel 1 상한 유지, 공유 schema/Cargo 변경 없음.

## 인수 조건

없는 root→생성→재연결, root identity 교체→재연결, native 변경 알림 회귀. watch unavailable 상태와 다른 UI 오류를 분리. 통합 검사와 가상 화면만 사용.

## 검증과 handoff

없는 root 복구·폴더 identity 교체·주기적 reconciliation·native write 회귀 통과. WSL 마운트에서 알림만으로 파일 반영이 실패한 사례를 재현하고 root mtime/30초 보완 후 가상 GUI 20파일 반영 확인. 실제 볼륨 제거/sleep 및 다른 OS는 미검증. 공통 fmt/clippy/test/release 통과. 전체 74 tests 및 실행 명령/측정은 [통합 검증](../validation/followup-validation.md)에 기록한다. 작업 범위 완료이며 S00 최종 스택 또는 전체 P0 완료를 뜻하지 않는다.
