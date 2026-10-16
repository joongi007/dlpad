# 세션 쿠키 importer 기반 검증

2026-10-04. 새 `crates/engine/src/cookies.rs` 및 `crates/engine/tests/session_cookies.rs`를 작성했다. root가 module export와 시험 활성화를 수행했다. 최초 Linux integration 11개 및 내부 secret Debug unit 1개가 실제 통과했고 engine all-target clippy도 통과했다. 제품 UI/실제 yt-dlp argv는 연결하지 않았다. R04 전체 완료가 아니다.

## 공개 계약

`import_session(source:&Path,owned_cache:&Path)->Result<SessionCookieLease,CookieImportError>`. lease의 `entries()`/`bytes()`는 최초 가져오기 통계, `path_for_tool()->&Path`는 정리 전·tool 종료 전만 사용할 민감한 임시 경로 capability다. 일반 job options/snapshot/DB/log/진단에 넣지 않는다. `cleanup(&mut self)`는 성공 시 반복 호출 가능하고 Drop은 best-effort 정리를 한다. 임시 복사본을 사용하는 모든 tool이 종료될 때까지 lease를 유지해야 한다. 중간 교체·정리 실패는 고정enum으로 반환하고 원문 IO 오류/path는 전파하지 않는다.

Sourcepath는 lease에 저장하지 않는다. SessionCookieLease/SensitivePath/SensitiveBytes Debug는 고정redacted 값이고 Error Debug/Display에는 입력값·경로·행내용이 없다. 읽기scratch와 유효rawbuffer는 Drop 시volatile메모리소거를 한다. 실제 원문file contents를 영구store로 옮기는 기능은 없다.

## 입력과 비용

[공식 yt-dlp FAQ](https://github.com/yt-dlp/yt-dlp/wiki/FAQ#how-do-i-pass-cookies-to-yt-dlp)는 Mozilla/Netscape 형식과 첫 줄 `# HTTP Cookie File` 또는 `# Netscape HTTP Cookie File`을 요구한다. importer는 그 두 header를 받아 header/comments/LF/CRLF/rawbytes를 변환 없이 복사한다. `#HttpOnly_`행은 일반comment로 버리지 않고 cookie로 센다. 7필드/구분TAB6개, TRUE/FALSE flags, domain/flag일치, slash path, 정수 또는 빈expiry를 검사한다. nameless/빈value/header-only0entry는 허용한다. 이 검사 자체가 실제 계정 인증 성공을 뜻하지 않는다.

총1MiB/물리line16KiB(줄바꿈LF제외·CR포함)/cookie4096개가 상한이다. UTF8불량/NUL/잘못된field/control/초과는 거절한다. 원본regularfile만 읽으며 입력을 최대1MiB+1 읽은 시점에서 중단한다. transientVec capacity는1MiB+1, scratch는4KiB이고 parser의7개field는borrow한다. 전체stream을 무제한 적재하지 않는다. 추가Cargo dependency는 없다. 동기bounded파일I/O이므로 root UI는 off-thread에서 호출해야 한다. 네트워크filesystem의 시간상한까지 구현한 것은 아니다.

## Unix 소유권·정리

Caller가 절대경로의 앱전용cache를 제공한다. source/cache의parent를component별openat/O_NOFOLLOW/O_DIRECTORY/O_CLOEXEC로 연다. source는fstatat사전type/identity 확인 뒤read-only로 열고 regularfile·전후size/mtime/ctime을 확인한다. cache는uid/정확0700을 요구하고 그 안에 새0700 ownerdir 및O_CREAT|O_EXCL|O_NOFOLLOW 0600 copy만 만든다. 원본은write/delete/rename하지 않는다.

Lease는cache/owner/file FD와dev/ino를 가진다. cleanup은현재cache경로·ownerentry·fileentry와heldFD의identity/type/uid/mode·단일link를 확인한다. 변경되면삭제를거절한다. unlink성공 뒤fileFD를닫고 정확ownerdir만rmdir하며extra파일을재귀삭제하지 않는다. symlink는따라가지않고source/parentcache는삭제대상이아니다. 0700앱전용dir의정상소유계약과검사기반이며 같은UID의악성process가검사와unlink사이를동시에바꾸는경쟁까지원자적인conditionalunlink로방어했다고주장하지않는다. 그러한강한위협모델은후속설계가필요하다.

nonUnix는소유권/ACL을추측하지않고읽기전Unavailable이다. 실제Windows ACL·macOS 런타임·startup stale-owner 청소·argv privacy·session UI·tool이수정한cookiecopy 재검증은후속범위다. 복사본 정리 후에도 원본 사용자가내보낸cookiefile의삭제를 앱이 대신하지 않는다.

## 작성한 실패경로 시험

Unix integration11개와 내부secretDebug1개, nonUnixUnavailable1개를 작성했다. root가 정식 `.rs`로 활성화했다. 임시폴더·합성fixture만으로literal/HttpOnly/권한/cleanup/Drop/독립lease/세상한경계/잘못된header·fields·NUL·UTF8/source·cache symlink/FIFO/regular교체/owner·cache교체/foreignextra·mode변경을다룬다. 실제쿠키·browserprofile·계정·인증network는사용하지않는다.

실제 실행: `cargo test -p dlpad-engine --test session_cookies --locked` 11 PASS/0 FAIL ([결과](evidence/session-cookie-import/initial-test-result.json)), 내부 secret Debug unit 1 PASS ([로그](evidence/session-cookie-import/secret-unit.log)), engine all-target clippy exit0 ([로그](evidence/session-cookie-import/clippy.log)). 이 결과는 실행 전 검증 API 추가 이전 snapshot이다.

## 실행 직전 lease 검증 후속

`validate_for_tool(&self) -> Result<(), CookieImportError>`는 정리된 lease 및 현재 cache/owner/file의 identity/type/uid/permissions 변경, symlink/누락/추가 hardlink를 거절한다. 원래 파일 내용을 재파싱하는 기능은 아니며 tool이 정상적으로 같은 inode의 cookie 내용을 갱신하는 것은 허용한다. 외부 tool 실행 직전에 호출하고 종료까지 lease를 유지해야 한다. 동일 UID 동시 조작에 대한 원자적 보호 주장은 하지 않는다.

live/cleaned 상태, 세 permission 대상 변경과 복구, copy 누락/추가 hardlink 시험 3개를 추가하고 기존 copy/owner/cache 교체 시험에도 실행 전 거절 인수를 추가했다. 새 API의 Cargo 검증은 root 통합 검증 대기 중이다. 실제 Windows/macOS·계정 인증 인수는 여전히 미검증이다.

## 실행 종료 미확인 시 정리 보류

`defer_cleanup(&mut self)`는 owned 핸들을 take하고 자동 unlink 경로를 비활성화한 뒤 모든 File 핸들을 정상 Drop으로 닫는다. mem::forget이나 FD 누수를 사용하지 않는다. 0700/0600 복사본은 명시적 후속 복구를 위해 남으며 lease는 실행 검증에 실패한다. 이후 cleanup/Drop은 남은 복사본을 삭제하지 않는다. 원본·복사본·권한 보존 및 Linux의 해당 cache/owner/file 경로 FD 종료 시험을 추가했다. 새 시험은 root Cargo 검증 대기다. startup recovery는 root의 별도 설계이며 모든 오류 경로를 해결했다고 주장하지 않는다.
