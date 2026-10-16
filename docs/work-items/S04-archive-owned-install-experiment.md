# S04: 공식 archive 소유 설치·제거 실증

- 상태: in-progress
- 분류: experiment
- 우선순위: P0
- 요구사항: R05, R08, R11
- 담당/수정 범위: root — crates/engine/examples/install_managed_archive.rs, 이 문서와 통합 상태 기록
- 선행 작업: bounded archive parser, inactive install/fullset remove, streaming HTTPS worker

## 사용자 결과

공식 고정 package를 실제 크기로 임시 owned cache에 설치하고 원 inactive 세트를 제거한다. 제품 GUI 연결·공급 승인·runtime 활성화와 구분한다.

## 계약과 범위

절대 manifest 경로를 bounded64KiB로 읽고 validate한다. local 모드는 별도 절대 source를 read-only 설치하고 download 모드는 기존 제품 HTTPS worker를 호출한다. tempfile 소유 디렉터리만 사용하고 original 외부source 삭제를 하지 않는다. 설치 inactive/fullset digest 및 원 세트 제거와 elapsed/RSS를 boundedJSON으로 기록한다. 실행 파일은 실행하지 않으며 기존 사용자 cache/Documents/영상/인증에 접근하지 않는다. 새 dependency는 없다.

## 인수 조건

실제 archive와 component bytes/hash 일치, active 미변경, exactset 제거 및 cache 남은항목 검증. source보존은 local 원본의 전후SHA로 확인한다. 실패exit와 실제 전송/로컬 구분, Windows/macOS 미확인과 supplier trust/notice gate를 기록한다.

## 검증과 handoff

구현·실행 후 원 evidence를 추가한다. 계획이나 archive parser 이전 결과를 설치 완료 증거로 사용하지 않는다.

## 실제 실행 결과

release example build exit0. [로컬 결과](../validation/evidence/managed-archive-owned-install-first/local.json)와 [실제 HTTPS 결과](../validation/evidence/managed-archive-owned-install-first/download.json)는 공식 고정Deno2.9.7 archive41596794 bytes/component95830104 bytes 설치·fullset검증·inactive유지·exactset제거 PASS다. local source 전후SHA 동일, elapsed3.72초/peak4052KiB; HTTPS elapsed6.92초/peak5792KiB, 원 SHA c6527f24… 일치다. 각 실행 source snapshot 불변/원binary SHA를 기록했다. 도구실행/활성화/GUI/독립publisher trust·notice 채택/다른OS 실제공급 설치는 미검증이다.

최신 Linux 도구회귀는56PASS1FAIL1ignored: 기존 direct version timeout 시작확인에서 StartupUnconfirmed가 io::TimedOut으로 반환되어 일반Io로 전달됐다. root는 도구 timeout domain으로 정규화하며 테스트를약화하지않는다. Windows midstream 회귀는39PASS0FAIL0ignored이다. 후속 전체workspace를 별도실행한다.

## timeout 교정 재검증

io::TimedOut→ToolError::TimedOut 정규화 후 root Linux 도구 회귀57PASS/0FAIL/1ignored를 실제 확인했다. 정확한 기존timeout인수와 midstream read인수를 유지했다. 새 source 전체Linux/Windows workspace collector는 실행 중이며 terminal/sourceSHA 확인 전 전체PASS로 보고하지 않는다.
