# S02: Windows startup GUI cookie seed 예제

- 상태: done
- 분류: prototype
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: diagnostic_cache — /tmp/windows_cookie_startup_seed.rs 및 이 문서만
- 선행 작업: actual Windows public repository/cookie gate/guardian/public recovery; root build/collector

## 사용자 결과

GUI startup 실제 회수/보류/오류 검증을 위한 합성 seed Rust 예제를 제공한다. 제품 source/Cargo 변경이나 실제 seed 실행은 하지 않았다.

## 계약과 범위

CLI recovered|pending|error fresh_fixture_exe manifest_path. expected는 object가 아닌 STRING이다. 원 Windows TEMP 직계 자식 dlpad-windows-cookie-download-owned-*를 새로 생성하며 appdata=root/data, library=root/media, source=root/source.cookies, sentinel=root/foreign-sentinel, cache=root/data/session-cookies로 고정한다. appdata도 root와 동일하지 않고 내부 경로다. manifest version1/owned_root/appdata/library/source/sentinel/cookie_owner/source_sha256/sentinel_sha256/expected를 16KiB 이내 create_new에 기록한다. 실제 fixed synthetic 쿠키와 capability만 허용한다.

recovered는 actual DownloadRepository/execute_with_session으로 두 native helpers가 완료된 Downloaded를 확인한 뒤 lease.defer_cleanup한다. 원 journal 두 Resolved/256B receipts를 확인하지만 public recover를 미리 실행하지 않는다. pending은 실제 import pristine marker/empty receipts를 defer한다. error는 own fixture cache 디렉터리를 preserved-session-cookies로 rename해 원 session을 보존하고 원 startup cache 위치에 plain synthetic 파일을 작성한다. ACL/사용자 보안 설정 변경 없이 startup unsafe-directory 오류를 유도한다. sentinel은 cache 외부 root에 놓아 foreign entry 때문에 recovered 케이스가 pending으로 오염되지 않도록 한다.

기존 fixture tool은 root/private-cache만 허용한다. draft 주석의 root/data/session-cookies 고정 대안을 root가 실제 fixture에 연결해야 recovered seed가 실행된다. 반드시 exact cache equality/직계 owner(cookie-session-64lowerhex)/cookies.txt basename을 확인하고 기존 ORIGINAL TEMP directchild·cap·source fixed content·plain/reparse·bounded cookie content 조건을 유지한다. 임의 starts_with root 전체로 넓히지 않는다. seed 옆에 fresh dlpad-process-guard.exe를 두어 기존 sibling fallback을 사용한다. seed를 self guardian 등록하면 headless entry가 없으므로 금지한다.

## 인수 조건

root가 freeze 종료 후 실제 Windows build/run하며 각 manifest의 source/sentinel hash와 bounded paths를 확인한다. collector는 GUI 전 recovery를 호출하지 않고 정상 case cookie_owner 제거, pending owner 보존, error 정제 안내·원 session/sentinel 보존을 각각 검증한다. 임시 root는 명시 manifest ownership으로만 정리한다. 실제 사용자 쿠키·Documents·영상·desktop을 사용하지 않는다.

## 검증과 handoff

기존 fixture tool/실제 Windows cookie download gate tests/public recovery/앱 startup source를 읽고 draft 작성. rustfmt 임시 예제 exit0. Cargo/native seed/GUI는 실행하지 않았다. root 및 collector에게 manifest 계약을 전달한다. 초안 완료는 GUI 실제 검증 완료가 아니다.

Root가 Windows native example을 workspace에 통합했다. 다른 OS는 fixture 실행 시 명시 exit2이며 OS 검증을 주장하지 않는다. 합성 도구는 exact root/private-cache 또는 root/data/session-cookies의 직계 64hex owner/cookies.txt와 고정 내용만 허용한다. 독립 engine examples의 guard는 같은 build directory 후보로 찾고 등록 앱 우선은 유지한다. 새 Cargo 의존성은 없다.

## 실제 native 결과

first Clippy exit101: 고정 SourceUrl ValidationError는 std::error::Error가 없어 boxed ? 변환 실패. root가 고정 정제 map_err로수정하고자료보존. second all-target engine Clippy와examples/guardbuildexit0. recovered/pending/error seed3CLI실행exit0/sourceunchanged/inputdesktopunchanged. root 원manifest·source/sentinel SHA·strict5files·실제record/receipt를대조한 root-seed-review.json 생성: recovered Resolved2/256+256, pending/error Empty2/0+0. GUI실행전원사본은모두존재하며 publicrecover미호출. 실제GUI인수는미검증이고system/security전원설정변경없음.

## root 실제 private desktop 시작 복구 인수

최신 CPU release SHA6898fd11ea7debf031efd5ce8fc8b734356ad4720ff4b0aea44ccac7dc63daea와 독립 C# runner를 native argv로 실행했다. 기존 csc로 compiler exit0를 확인했으며 PowerShell 정책은 변경하지 않았다. 첫 recovered 실행은 verbatim Windows 경로의 .NET 정규화 오류로 앱 실행 전 실패했다. runner에서 원 드라이브 절대 경로를 정규화한 뒤 새 evidence에 재실행했다.

`windows-cookie-startup-gui-recovered-second`, `windows-cookie-startup-gui-pending-first`, `windows-cookie-startup-gui-error-first`의 collector exit0와 GUI result PASS를 확인했다. 완료 receipt 원 소유자 회수, pristine 미완료 소유자 보존 안내, unsafe cache 시작 차단을 각각 실제 native UIA로 확인했다. 세 recovery.png를 주 담당이 직접 검토했다. error 화면은 canonical 경로의 원 PNG(SHA c36b5e72384d66fadab6ae1589ec774e689136851845dfa84dfbf9b496cc68fb,68181bytes)를 다시 열어 footer의 고정 오류 안내와 재확인 버튼이 실제 viewport에 보임을 확인했다. 첫 alias 경로 이미지 표시와 구분하고 원 파일 SHA 및 canonical 재검토를 근거로 한다. URL/쿠키 입력은 비어 있고 다운로드를 자동 시작하지 않았다. 원 source/foreign SHA 보존, input desktop 불변, WM_CLOSE 정상 종료, 원 소유 Job empty 및 private desktop close를 확인했다.

최신 Linux 기본/CPU workspace collector terminal exit0와 전체 mandatory 명령 PASS, 현재 source_final SHA 불일치0도 확인했다. 이 세 startup fixture는 실제 GUI appkill·busy Bound·partial deletion·same-app 인증 다운로드·IME·전체 OS 인수를 대신하지 않는다.
