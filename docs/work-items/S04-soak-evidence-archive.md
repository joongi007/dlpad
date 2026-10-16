# S04: 영구 소유 soak 증거 archive

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R08, R10
- 담당/수정 범위: soak_recovery_audit; scripts/soak_evidence_archive.py, scripts/test_soak_evidence_archive.py, 이 workitem, docs/validation/soak-evidence-archive.md
- 선행 작업: S04-soak-interruption-audit-20261003

## 사용자 결과

임시 collector 소실 뒤에도 소유 합성 시험의 진행·중단·terminal 증거를 프로젝트에 bounded 보관한다.

## 계약과 범위

실제 soak launch는 root 담당이다. 이 작업은 collector와 합성 시험만 변경한다. source/Cargo/공유 인터페이스 변경과 실제 시험 실행·프로세스 signal·재시작은 없다. explicit root/boot/PIDstart manifest와 새 project evidence destination만 받으며 symlink/ownership mismatch는 실패한다.

## 인수 조건

고정 allowlist, 파일당1MiB/전체8MiB/history512, 64KiB streaming SHA, atomic publication. 현재 boot/PIDstart와 terminal 결과를 구분하고 과거 running을 현재생존으로 간주하지 않는다. symlink/손상/oversize/프로세스재사용/boot변경/실패terminal을 합성 테스트한다.

## 검증과 handoff

scripts/soak_evidence_archive.py와 전용 unittest를 구현했다. python3 -m unittest discover -s scripts -p test_soak_evidence_archive.py -v: 23개 PASS. 구현·실행 계약·미검증 사항은 docs/validation/soak-evidence-archive.md에 기록했다. 실제 soak launch/프로세스 종료/제품 변경은 없으며 실제8시간 완료와 OS별 검증은 남는다.

## 통합 종료 경계 보완

root 통합 검토에서 두 그룹 시작 시각 차이로 정상 guardian 종료를 interruption으로 오인하는 경계를 발견했다. scope별 완료 증거와 고정 process role 계약을 추가하고, 증명된 scope의 종료는 expected_terminal_exit로 표시한다. 다른 scope의 미완료 PID 부재는 여전히 interrupted다. guardian-first/download-first, split 이후 전체완료, invalid terminal, unexpected death, unknown role 회귀를 포함한23개 테스트가 PASS(1.148초)했다. 실제 soak는 아직 실행하지 않았다.
