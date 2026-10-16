# S03: public process::run startup child cleanup 검증

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R08
- 담당/수정 범위: soak_recovery_audit; 신규 scripts/public_startup_helper_probe.py, 이 work-item, 전용 validation/evidence
- 선행 작업: root 새 startup-wait public/단위/clippy 및 helper/driver READY

## 사용자 결과

공개 process::run에서 helper가 EP를 보내거나 R 없이 멈춰도 정해진 startup wait/owned kill/reap 후 오류를 반환하는지 확인한다.

## 계약과 범위

현재 executable env override가 없으므로 소유 /tmp isolated folder에 실제 ignored driver binary를 복사하고 adjacent dlpad-process-guard만 전용 synthetic Python fixture로 둔다. 기존 adjacent executable 선택 계약을 사용하며 원본 binary/helper는 수정하지 않는다. fixture는 child PID/start/socket EOF를 기록하고 EOF 후에도 잠들어 natural5s→owned kill2s 경로를 실제로 검증한다. manager-independent fixture이고 실제 service cgroup은 root 새 strict13+startup4 회귀에서 별도 확인한다. 제품/Cargo/global/user desktop/8h/다른 프로세스는 변경하지 않는다. READY 이전 실행하지 않는다.

## 인수 조건

EP-sleep: exact fixed EP 오류 + socket EOF + 약5초 cleanup + helper/driver reaped/FD동일. no-R: 실제 readiness10초 timeout + socket EOF + natural5초/ownedkill finite cleanup. 외부 synthetic process sentinel 보존, 소유 pidfd만 fallback cleanup, 로그/결과 상한/부모23초 deadline. 관찰되지 않은 exact identity 또는 EOF는 PASS 금지.

## 검증과 handoff

root READY 후 manager strict13+startup4 실제17case 및 공개run copied fakehelper EP/noR2case 모두exit0/PASS. 공개 EP5.054s/noR15.127s에 실제 socketEOF→natural5s→ownedkill/reap, FD4→4/helperowner exactidentity부재/도구미시작/소유sentinel과tmpcleanup 확인. manager17의serviceGC13cgroup도별도검증했다. 상세명령/source/raw/제약은 guardian-startup-wait-regression validation에 보존했다. 공유 제품/Cargo/기존17원시/8h변경없음.
