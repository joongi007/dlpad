# S02: 검증된 reader receipt로 같은 boot의 세션 회수

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: root — cookies.rs recovery/공유문서; metrics_recovery — cookies/reader_journal.rs; stress_recovery — tests/cookie_recovery_gate.rs
- 선행 작업: 제품 DownloadRepository 실제 gate4PASS, exact anchored journal/receipt 검증

## 사용자 결과

앱 중단 후 같은 boot로 다시 시작할 때 실제 reader 종료 receipt가 있는 private 복사본만 회수한다. 아직 실행 중이거나 증거가 미완성인 복사본은 보존한다.

## 계약과 범위

기존 recovery record1/2를 그대로 읽는다. empty journal은 같은boot 정리 근거가 아니다: 이전 ungated제품의reader는journal Intent없이실행될수있었으므로보존한다. 원래 Bound/Resolved 슬롯1개이상과모든slot의실제receipt 검증·resolve sync가 필요하다. active recordflock/Intent/partial/wrongreceipt는보존. PID/unit명/mtime/ENOENT를 proof로추측하지않는다. nativeidentity교체는거절한다. 이전boot기존정책 유지.

## 인수 조건

실제 manager metadata/download뒤 앱프로세스exit 시sameboot회수, download실행중 이번앱만kill/reap뒤guardian EOFcleanup+nativeBoundreceipt로bounded회수. source/sentinel/완료artifact 보존. empty/Intent/partial/mismatch/active lock preserve 실패 경로. 오래된record 호환과전체engine회귀 유지.

## 검증과 handoff

구현/fixture준비중. 모듈의합성receipt unit은actualnativeproof를대체하지않는다.

root helper build exit0, engine library247PASS/2ignored. 첫fixture compile는Arc타입1error, 두잘못된filter실행은0tests(인수아님), 첫actual2case는foreignsentinel의기존rejected1을cookie실패로취급한수집기조건으로2FAIL. worker전 baseline(0,0,1)과bytes를먼저검증하여cookie회수조건을약화하지않고분리했다. 원시전부보존. 정확 `cargo test -p dlpad-engine --test cookie_recovery_gate --locked recovers -- --ignored --nocapture --test-threads=1` 실제2PASS(4.85초): liveappkill뒤firstdeferred1에서terminalreceipt회수, success앱destructor생략뒤recovered1. [원시와 source/helper SHA](../validation/evidence/cookie-sameboot-product-first/source-and-helper-sha256.json). 이전bootpending 인수는별도발견된미완료이며이작업의기존정책유지조건이아직완료되지않아in-progress를유지한다.
