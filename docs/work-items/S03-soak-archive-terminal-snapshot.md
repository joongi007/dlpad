# S03: Archive terminal snapshot race 수정

- 상태: completed
- 분류: bug
- 우선순위: P0
- 요구사항: R08, R10
- 담당/수정 범위: soak_recovery_audit — scripts/soak_evidence_archive.py, scripts/test_soak_evidence_archive.py, 이 문서
- 선행 작업: 실제8h terminal 독립 감사와 root 원시검증

## 사용자 결과

Source terminal이 읽기/복사 사이 출현하면 실제 보존된 completed proof로 판정한다. 진짜 미완료·실패·소유 프로세스 예상 밖 종료는 그대로 구분한다.

## 계약과 범위

판정은 현재 step에서 보존한 allowlist terminal snapshot만 사용하고 이후 exact PID/start를 관측한다. 기존 archive·launch·실제8h evidence를 덮어쓰지 않는다. Exit0/finite28800초/명시 eight-hour/자손 정리 proof를 완화하지 않는다. Rust/Cargo/GUI/8h 실행과 프로세스 signal은 없다.

## 인수 조건

- Copy 도중 runner terminal 출현을 deterministic mock으로 재현하고 copied bytes/proof/state 일치를 검증한다.
- Current step 미복사·omitted·invalid terminal은 이전 보존 파일을 새 proof로 사용하지 않는다.
- Copy 중 진짜 owner death와 incomplete proof는 interrupted이고 completed group의 예상 종료만 허용한다.
- 기존 모든 bounded/path/atomic/split-completion 회귀를 유지한다.

## 검증과 handoff

Red 재현: `python3 -m unittest discover -s scripts -p test_soak_evidence_archive.py`는26개 중2FAIL/exit1이었다. Runner terminal이 copy 도중 나타나면 state interrupted였고, copy 중 owner가 사라지면 copy 이전 live 관측 때문에 watching이었다.

수정은 allowlist를 먼저 보존하고 현재 step의 성공적으로 복사된 terminal 파일만 bounded JSON으로 읽도록 바꿨다. 읽은 bytes SHA가 copy manifest SHA와 같아야 한다. Missing/omitted source는 예전 dest terminal이 남아 있어도 새 proof로 쓰지 않는다. Proof 판정 이후의 원본 source 변경은 보존된 snapshot과 별개이며 분석은 실제 보존 bytes를 기준으로 한다. Terminal snapshot file size/SHA를 result에 명시한다. Copy 뒤 boot를 다시 읽고 exact PID/start를 관측하므로 미완료 scope의 진짜 예상 밖 종료는 interrupted다. Boot mismatch면 proof를 버리고 interrupted를 유지한다.

Green 최종 같은 명령은30PASS/0FAIL/2.901초/exit0이었다. Late terminal/current-step missing/omitted/stale copy/source-after-copy 변경/copied hash tamper/nonzero late runner exit/actual death 경계와 기존 split-completion/finite8h/path/symlink/FIFO/atomic cleanup/byte bound 회귀를 포함했다. 수치28801은 합성 proof 값이며 실제 장시간 실행은 하지 않았다. Test fixture temp와 신규 synthetic evidence만 tearDown에서 정리했고 실제8h archive·launch·terminal evidence는 손대지 않았다.

Source 동결: soak_evidence_archive.py SHA `568b757b97607b976250f03e09c64e35ab388e8fd51b8e324b1a90011c8e4387`, test_soak_evidence_archive.py SHA `4c76ff7bc0dbfa49107c5738c90bef8eb03c6f48d5921c8ce3a3ba227e397d42`. Exit0/finite28800/명시 eight-hour/guardian descendants_reaped proof는 변경하지 않았다. Rust/Cargo/GUI/프로세스 signal/실제 soak 재실행은 없다. Root가 source와30case를 독립 검증한다. 기존 actual interrupted archive는 과거 실행 그대로 유지하고 다음 새 collector 실행은 fresh destination을 사용해야 한다.

Root 독립 source검토와 실제 회귀30개 PASS(2.519초/exit0). 첫 sandbox시도는 프로젝트 case경로가readonly라29error였으며 원시를 보존했고 승인된 프로젝트쓰기 범위 승격 재실행으로 통과했다. 기존 8시간 evidence는 변경하지 않았다. 독립원시: docs/validation/evidence/soak-archive-terminal-snapshot-root/.
