# S03: 실제8시간 terminal 독립 감사

- 상태: completed
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R10, R11
- 담당/수정 범위: soak_recovery_audit — 이 문서와 신규 terminal evidence만
- 선행 작업: 고정 launch/ownership 및 기존 archive interrupted 관측

## 사용자 결과

Guardian/download/runner/archive terminal을 독립 대조해 실제 완료와 archive 수집 race를 구분한다. 기존 archive는 덮어쓰지 않고 완료 proof가 있는 원시 자료만 새 evidence에 보존한다.

## 계약과 범위

원본 owned temp와 project evidence를 읽고 host boot/PID/start를 정확 대조한다. 프로세스 종료·재시작·Rust/Cargo/P0 status 수정은 없다. DB terminal과 소유 worker 종료 증거를 확인하기 전 DB quick_check/쓰기 연결은 금지한다. 원인 불명은 추정 완료로 대체하지 않는다.

## 인수 조건

- 실제28800초 이상·runner exit0·terminal source/binary provenance·자손 정리를 독립 확인한다.
- 기존 interrupted archive proof와 새 늦은 terminal proof를 구분하고 source/size/SHA create-only 보존한다.
- Terminal 이후 SQLite read-only integrity·최종 job/attempt 상태를 검증한다.

## 검증과 handoff

승격 host 읽기2026-10-03T20:08:54.280693Z는 launch boot와 동일하고 ownership10개+archive owner를 포함한 소유11 PID가 모두 absent임을 확인했다. Sandbox 조회로 부재를 단정하지 않았다. Signal/restart는 실행하지 않았다.

Original download terminal은 completed/elapsed28801.956255879초/eight_hours_verified=true/Attempt1/artifact17B였다. Original runner terminal은 completed/elapsed28805.355041029초/exit0이며 test_result가 download terminal과 정확히 같았다. Guardian terminal은 completed/elapsed28804.0931156초/eight_hours_elapsed=true/descendants_reaped=true/480samples였다.

기존 archive-result는20:03:26.446027Z interrupted/guardian scope completed/download scope null을 유지한다. 그런데 그 archive의 runner-result.json은 실제 completed/exit0였고 파일 manifest SHA와도 일치했다. Collector step source는 results/proof/state를 먼저 읽고 이후 ALLOWLIST file을 복사한다. 따라서 먼저 읽을 때 아직 없는 runner terminal과 나중 복사 때 출현한 terminal이 다른 snapshot에 들어가는 read/copy race의 직접 근거가 있다. 기존 interrupted를 completed로 덮어쓰지 않는다. Collector 종료 grace/re-sampling 회귀는 root가 별도 task로 처리한다.

Terminal과 소유11 PID 부재 이후 원 DB를 mode=ro&immutable=1로 열었다. WAL/SHM 없음, PRAGMA integrity_check=ok, 최종 long-download job은 Downloaded/revision4/Attempt1이었다. 원 DB SHA는 읽기 전후 동일했다. Artifact bytes는 정확히 `synthetic-final-1`17B이며 partial fixture는65536B다. 이 partial 보존은 합성 fixture 계약이며 사용자 파일이나 제품 미정리로 확대 해석하지 않는다.

Immutable download_soak/helper/CPU의 launch size/SHA/device/inode를 모두 직접 대조했다. scripts/download_soak.py, long_job_soak.py, soak_evidence_archive.py 및 tests/download_soak.rs SHA도 launch source_hashes와 일치했다. 현재 변경된 다른 Rust source를 당시 실행 binary로 간주하지 않는다. CPU SHA166946…/download_soak112cfd…/helper1328f3… scope만 인정한다.

새 [독립 terminal result](../validation/evidence/soak-terminal-independent-20261004/result.json)에 launch/ownership/original archive 두 파일/세 terminal/result·testlog·samples·guardianlog/SQLite/artifact/partial/synthetic tool/extraction/control 총17개 allowlist 파일813205B를 create-only 보존했다. 파일당1MiB/전체8MiB 및64KiB streaming SHA를 적용하고 source/copy SHA 일치·원시 archive SHA 불변을 확인했다. 기존 archive manifest13개 파일의 size/SHA도 검증했다. 새 result는 DB final job·11 identity 관측·binary/source provenance·복사 manifest를 포함한다.

실행 명령은 승격 read-only Python 원본 terminal/PID 대조, terminal 이후 sqlite3 URI read-only 검사, 명시 allowlist create-only raw 보존이었다. 모두 exit0이며 프로세스 조작·Cargo·제품 source·P0 status 파일 변경은 없다. Root가 새 evidence를 독립 검토한 뒤 장기 gate scope를 갱신한다. 실제8시간 Linux headless synthetic engine+SQLite/guardian 범위만 완료했으며 GUI·실제 영상·외부 네트워크·인증·새 user-systemd backend8h까지 통과했다고 주장하지 않는다.
