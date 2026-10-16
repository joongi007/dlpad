# S02: 세션 쿠키 파일 가져오기 기반

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: stress_recovery — crates/engine/src/cookies.rs, crates/engine/tests/session_cookies.rs, 본 작업 기록 및 전용 validation
- 선행 작업: root 공유 API 승인; module export/argv/UI 연결·Cargo 검증은 root 담당

## 사용자 결과

Netscape 쿠키 파일을 원본 변경 없이 세션 전용 복사본으로 읽고, lease 종료 시 정확한 소유 파일만 정리하는 기반을 제공한다. 아직 다운로드/UI와 연결하지 않아 R04 전체 완료가 아니다.

## 계약과 범위

AGENTS/INTEGRATIONS의 사용자 전용 임시 쿠키·비밀마스킹 계약. 공식 yt-dlp FAQ는 첫 줄 '# HTTP Cookie File' 또는 '# Netscape HTTP Cookie File'을 요구한다. seven fields=TAB구분7필드/구분TAB6개로 해석하며 #HttpOnly_을 지원하고 입력의 header/newline/comment/rawbytes를 그대로 보존한다. 총1MiB/물리line16KiB/entries4096 상한, NUL/UTF8불량/형식불량/초과를 거절한다.

API: import_session(source:&Path,owned_cache:&Path)->Result<SessionCookieLease,CookieImportError>, lease.entries()/bytes()/path_for_tool()->&Path/cleanup(&mut self). path_for_tool은 민감한 tool argv capability이며 일반 snapshot/DB/log/진단에 기록하면 안 된다. Debug/error는 원문·모든path를 숨긴다. 원sourcepath는 lease에 저장하지 않는다. 원data 임시buffer도 redacted Debug와 Drop 메모리소거를 사용한다.

Unix에서 절대path 구성요소를 anchored openat/O_NOFOLLOW로 열고 caller의 앱전용cache uid/0700을 확인한다. 그 안에 신규0700 ownerdir와 create-new0600 file만 만들고 parent/dir/file FD identity를 보유한다. cleanup은 파일/owner entry identity·type/uid/mode 변경 시 거절하며 다른 파일을 재귀삭제하지 않는다. 앱과외부tool의 정상정리 계약이며 동일UID 악성process의 동시조작까지 OS원자적 conditional-unlink를 보장한다고 주장하지 않는다. nonUnix는 permissions/ACL검증 없이 Unavailable. source/lib/core/options/UI/Cargo/schema는 수정하지 않는다.

## 인수 조건

임시폴더·합성cookie만 사용한다. LF/CRLF/HttpOnly/literal보존/Debug마스킹과 명시cleanup/Drop/source보존을 확인한다. malformed/NUL/oversize/line/entries 경계, symlink/file교체/owner교체/외부sentinel 보존/비공개cache거절을 실패경로 시험에 포함한다. 공유Cargo검증은root가 수행하고 실제실행결과에만 PASS를 기록한다. 비Unix는 안전한Unavailable 시험이며 실제WindowsACL검증으로 쓰지 않는다.

## 검증과 handoff

작성 중. 실제 계정/쿠키/인증network는 사용하지 않는다. 새 deps없음, secret버퍼상한1MiB+1/4KiB읽기scratch, parser필드borrow, 임시파일최대1MiB. source freeze/API보고 후 root통합검증을 기다린다. startup stale-owner 청소, argv redaction, session UI연결과 실제OS별ACL은 후속수직통합이다.

## root 모듈 연결과 최초 실제 시험

root가lib.rs에cookies export를추가하고pending시험을최종session_cookies.rs로전환했다. `cargo test -p dlpad-engine --test session_cookies --locked` 실제11PASS/0FAIL/0ignored(0.02초). [결과](../validation/evidence/session-cookie-import/initial-test-result.json). Linux합성fixture기반이며nativeUI/인증다운로드/실제쿠키/WindowsACL인수는아니다. 내부 secret Debug unit 1 PASS와 engine all-target clippy exit0도 실제 완료했다. 로그는 전용 evidence 폴더에 보존한다.

## 실행 직전 검증 API 확장

root 요청으로 `validate_for_tool(&self) -> Result<(), CookieImportError>`를 추가한다. 변경 범위는 cookies.rs/session_cookies.rs와 전용 문서만이며 root가 실행/UI 연결을 담당한다. 정리된 lease·cache/owner/file 교체·권한 변경·누락·hardlink 증가 거절과 정상 lease 인수가 목표다. API 추가 후 Cargo는 root 검증 대기 중이다. 원문 재파싱이나 동시 동일 UID 공격의 원자적 방어는 포함하지 않는다.

## 실행 종료 미확인 시 정리 보류

`defer_cleanup(&mut self)`는 owned 핸들을 take하고 자동 unlink 경로를 비활성화한 뒤 모든 File 핸들을 정상 Drop으로 닫는다. mem::forget이나 FD 누수를 사용하지 않는다. 0700/0600 복사본은 명시적 후속 복구를 위해 남으며 lease는 실행 검증에 실패한다. 이후 cleanup/Drop은 남은 복사본을 삭제하지 않는다. 원본·복사본·권한 보존 및 Linux의 해당 cache/owner/file 경로 FD 종료 시험을 추가했다. 새 시험은 root Cargo 검증 대기다. startup recovery는 root의 별도 설계이며 모든 오류 경로를 해결했다고 주장하지 않는다.
