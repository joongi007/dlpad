# S04: 손실된 장시간 시험 재실행과 영구 증거

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R08, R10
- 담당/수정 범위: guardian_fault; 전용 workitem/validation 및 docs/validation/evidence/soak-restart-20261002/ 한정
- 선행 작업: DownloadRepository 합성 soak와 guardian release harness

## 사용자 결과

이전 /tmp 증거·프로세스가 사라진 장시간 시험은 완료로 처리하지 않고, 현재 snapshot에서 새 8시간 합성 시험을 실행한다. 진행 증거를 프로젝트에 bounded 보관해 /tmp 소실에도 결과 공백을 식별한다.

## 계약과 범위

AGENTS/README 문서 순서와 PRODUCT/CONTEXT/R04/ADR-0002를 다시 읽었다. source/Cargo/shared 문서 수정 금지. 정상 Git이 없으므로 기존 .git을 보존한다. 현재 snapshot을 빌드·짧은 실제 baseline 확인 후 scripts/download_soak.py로 fresh owned /tmp에 실행한다. 사용자 desktop/Documents/실제 미디어/네트워크는 없다. 새로운 실행은 이전 완료를 증명하지 않는다.

## 인수 조건

- 기존 증거·PID 부재와 현재 boot ID를 영구 JSON에 기록한다.
- 신규 binary/source hash, 명령, exec session, 실제 PID/start ticks와 주기적 progress/RSS/CPU 증거를 프로젝트에 남긴다.
- 각 proof 파일 1MiB 이하, 이번 합성시험 로그만 허용하고 외부 자료·비밀 복사 금지, 전체8MiB/512행 상한.
- 실제 28800초 이후 repository/artifact/DB reopen assertion이 통과해야 완료로 기록한다.

## 검증과 handoff

boot ID794988eb-539b-437e-8396-330738b565bb, 기존 두 /tmpstatus 및 PID29575/29577/103171/103274/103276 부재 확인. 이전 결과는 unverified이며 새로운 시험 착수 준비 중이다.

## 실제시작과handoff

현재shortbaseline1passed/1ignored(7.15초),helperbuildexit0. 새DownloadRepositorysession13723/releaseguardiansession64092 시작. 매분projectboundedarchive session21052 실행중. 정확source/binary/PID/start와부팅ID는 docs/validation/evidence/soak-restart-20261002/launch.json 및 전용 docs/validation/soak-restart.md에 기록. 아직8시간완료아님.
