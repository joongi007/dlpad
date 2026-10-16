# S07: 설치기 초기 검사 중단 후 재검사 복구

- 상태: done-scoped / 사용자 재검사 확인 대기
- 분류: fix
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: Root — crates/app/src/installer_main.rs, installer_catalog.rs, 이 기록, CONTEXT.md, README.md, docs/development/windows-user-preview.md와 현재 P0/검수 문서; p0_audit — .local-tools/session-recovery-20261013/install-cancel-retry-20261017/ 격리 GUI 재현 helper 전용; field_reveal — .local-tools/session-recovery-20261013/ui37-integration-prep/ 전체검사·패키지 helper 전용
- 선행 작업: v35 기존 설치 readiness, v36 whole; ADR0009

## 사용자 결과

설치기 최초 위치/기록 탐색을 중단한 뒤 재검사하면 같은 원 설치·기록을 다시 찾아 실행 준비로 복귀한다. 중단·재시도는 원 파일/기록을 삭제하거나 생성하지 않는다.

## 계약과 범위

초기 admission과 명시 Verify의 연결·늦은 worker 결과·revision·취소 ACK를 확인한다. shared schema/Cargo/lock/engine 변경은 Root 조정 없이 금지. 제품 GUI는 Rust native, 자동 검증은 private desktop 전용, 실제 사용자 자료는 사용하지 않는다. 치지직 정상 사용자 보고는 사용자 관측으로 기록하고 자동 저장 시험 성공으로 바꾸지 않는다.

## 인수 조건

- 초기검사→중단→worker 종료→재검사가 원 기록을 재발견하며 정상 설치는 실행 준비가 된다.
- 빈 경로·ambiguous/wrong journal·취소 직후 재요청·입력변경의 stale 결과는 실행 권한을 만들지 않는다.
- 원 payload/journal의 bytes·identity를 보존하고 UI에서 검사의 중단과 재시도 상태를 알린다.
- Windows 회귀, fmt/clippy/test/release와 필요한 격리 GUI를 Root 통합 검증한다.

## 검증과 handoff

실제 Windows 재현·수정·원자료 보존·새 설치 ZIP 제공·문서 갱신을 완료했다. 치지직 정상 추정 사용자 관측도 현재 P0 문서에 반영했다. 전체 P0 완료 아님.


## 원인과 실제 검증

203이 초기 discovery를 우회해 미확정/이전 base journal을 직접 검사했다. 202취소 후 초기 admission Unknown이 남아 새 위치 설치가 막히고, 실제 suffix-1 기존 설치에서도 SOURCE_CHANGED가 됐다. packaged 재검사는 현재 worker ACK 후 discovery를 재수행하며 explicit4path 직접검증은 유지한다. 취소 ACK 완료 안내와 작업별 취소 라벨을 추가했다.

private nativeState regression RED0/1(exit101)→GREEN1/0, 최종 GREEN은 ACK전중복차단과 반복재검사·원bytes 보존까지1/0. whole37 네 Cargo 검사0·1685/0/43ignored·43harness·source675·lock불변. 별도 실제 native mixed35payload/36→37installer baseline은 SOURCE_CHANGED/runoff, green은 suffix-1/runready로 동일 순서 복귀했다. 양쪽 compile/native0·설치기2개자연0/Job0·원5+5journal/payload5·private/input복구. Root원로그/pins와PNG3장직접검토PASS. [검수](../validation/ui-review-v37-20261017.md), [native검토](../validation/evidence/installer-cancel-retry-root-20261017/root-review.json).

설치기+512B, CPU/CLI동일, 새의존성/상주worker없음. 패키징6단계 실제0·ZIP전체CRC/SHA/graph258/notice456 검토 완료. `python3 scripts/check_docs.py` 실제 exit0: 1208 documents PASS/133 hash-matched packaged upstream notices 보존. 외부 URL·heading anchor는 검사 범위 밖이다. 새37bundle 전체실행·실제userdesktop·전체P0완료로표시하지않는다.
