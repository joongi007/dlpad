# S02: 합성 session cookie 다운로드 통합시험

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R08, R10
- 담당/수정 범위: soak_recovery_audit; crates/engine/tests/session_cookie_download.rs 및 이 work-item
- 선행 작업: metrics DownloadSession/execute_with_session/resume_with_session, stress SessionCookieLease validation

## 사용자 결과

합성 Netscape cookie의 private session 복사본만 metadata/download에 전달하고 version·로그·반환 metadata·DB에 인증 자료를 남기지 않는지 검증한다.

## 계약과 범위

0700 소유 temporary cache와 fake cookie 값/yt-dlp만 사용한다. 실제 계정/네트워크/자격 증명은 사용하지 않는다. 기존 download fixture와 lease ownership 계약을 따르며 제품 download/cookies/Cargo/schema/UI는 수정하지 않는다. caller가 lease를 보유하고 worker 종료 후 명시 cleanup한다. source와 unrelated cache sentinel 보존, cleaned/replaced lease failclosed를 시험한다. 다른 담당 source 작성 중에는 Cargo 실행하지 않는다.

## 인수 조건

version에 --cookies 없음, metadata/download에 같은 private path와 원문 bytes, 정상 artifact 검증, 민감 stdout/stderr와 자유 metadata 제외, SQLite/job/options snapshot cookiepath/value 미포함. metadata/download 실패·cancel 이후 caller cleanup와 source/sentinel 유지. cleaned/replaced lease는 tool 미시작. Failed resume는 새 session lease 경로로 재추출/다운로드하고 options snapshot에 남기지 않는다.

## 검증과 handoff

기존5개 시험은 root 실행 보고에서 engine lib/session-cookie importer와 함께 exit0였다. 이후 typed Summary.duration Option<f64> 경계를 확인해 sixth string/object secret echo→fixed redacted JSON Policy/no download/DB·source·sentinel 보존 회귀를 추가했다. 소유 rustfmt 완료 후 root가 통합 검증했다. Root의 전체 7명령 default/CPU 로그에서 session_cookie_download 6 PASS를 독립 확인했다는 보고를 받았다. Root 보고 기준 전체 default246/CPU387 및 양쪽7ignored, 모든 명령 exit0이며 이 시험 파일에 실제 계정·네트워크·GUI는 사용하지 않았다. [통합 실행 evidence](../validation/evidence/session-cookie-workspace/)를 기준으로 한다.

## 후속: 실제 guardian SIGKILL과 보존된 lease

Root가 담당한 종료 확인 실패 정책과 stress 담당 durable cookie recovery record에 대한 seventh 회귀를 추가한다. 수정 범위는 기존 소유 test/work-item만이며 제품 소스를 바꾸지 않는다. 기존6개는 보존했다.

fake download가 자기 PID/start와 guardian PPID/start를 소유 atomic JSON에 기록한다. Rust trigger는 worker argv가 합성 도구인 점과 실제 PPID를 확인하고 /proc start 재검증 전후에 pidfd를 연 뒤 정확한 guardian만 SIGKILL한다. direct fixture에서 남을 수 있는 worker는 300ms 뒤 독립 pidfd cleanup하며 PID/start 소실과 descriptor exit를 최대5초씩 확인한다. 이 정리를 adapter의 종료 확인 증거로 오인하지 않는다.

인수는 실제 `SessionCleanupDeferred`, Downloaded 미성립, secret/path 비노출, caller `defer_cleanup` 후 copy/recovery.json/owner.marker 보존, lease 재사용 거절, 해당 소유 디렉터리를 가리키는 모든 FD 닫힘, Drop 후에도 copy 보존, 원본·무관 sentinel 보존이다. 테스트 fixture의 마지막 temporary-directory 정리는 실제 worker/guardian 소실 확인 뒤 수행한다. 소유 rustfmt 완료 및 소스 동결 후 root가 Cargo를 실행했다. [실제 root 로그](../validation/evidence/session-cookie-recovery-first/download-fault.log)에서 기존6개와 seventh 모두 PASS, [결과](../validation/evidence/session-cookie-recovery-first/result.json)에서 exit0를 확인했다. Root는 회복 기록/UI gate를 포함한 CPU workspace all-target clippy도 exit0 보고했다. 소유 test 구현 완료이며 실제 GUI의 최신 회복 인수와 다른 OS는 별도 범위다.
