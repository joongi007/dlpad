# CPU 최대 유효 보고서 stress 근거

## 최초 조사

core MAX_REPORT_BYTES=2MiB는 방어 상한이다. 실제 값은 logs128×4096B, MPD8첨부×256nodes, range/mediaRange48B, pseudonym URL≤[url-128], depth≤63이며 원본 MPD는 최대1MiB이다. control/backslash/긴opaque문자열은 로그 정제에서 제외된다. JSON 최대 escape는 허용 quote 반복의 약2배이다.

기존 compiled core rlib를 사용한 독립 Rust scratch에서 실제 add_log/add_mpd_attachment API만으로 128개의4096B quote-heavy 로그와8첨부×256개의 최대range Initialization을 생성했다. JSON1841853B, Markdown1933100B를 확인했다. 원본 MPD는 첨부당55337B이다. URL pseudonym을 index128로 늘릴 여유는 약8192B이며 여전히2MiB 아래이다. 이 결과는 최대값의 수학적 증명이 아닌 경계형 유효 fixture 실증이다.

원시 소스/명령/크기는 evidence/cpu-report-stress/core-maximal-fixture.rs와core-sizing.txt에 보존했다. GUI 실패 경로 process evidence는256KiB라 현재 즉시 이 크기 report를 만들 수 없다. root에 명시 fixture-only 고정 builder 계약을 제안했다. 제품 수정/새 GUI stress 검증은 아직 수행하지 않았다. residual_risk Boxleak unit의2MiB초과를 GUI 성공으로 보고하지 않는다.

## WSL 중단 뒤 재개 조사 (2026-10-03)

기존 full-1/2 실패와 full-3 불완전 자료를 보존하고 frozen CPU SHA `3a1bded7e3e89074f2ba86124074aa32777cae512b81f1b69b4898539ffd728a`로 full-4~8을 실행했다. full-4는 시작 footer center y561을 harness가 y<550으로 잘못 제한한 오류였다. footer 영역을 실제 PNG 기준 y<580으로 보정했다. full-5에서 물리 header 클릭의 scroll animation 경합을 관측해 이후 버튼은 visible extents 확인 후 실제 AT-SPI Action.DoAction으로 실행했다.

full-6~8은 실제 합성 job Failed/Attempt 1과 열린 header까지 확인했으나 copy 버튼에 접근하지 못해 FAIL이다. full-7/8의 bounded extents trace에서 copy y593→526 이후 반복 outer wheel에도 y526이 유지되어 center y542가 content clip y535 밖이었다. full-8은 중첩 editor를 피한 outer right margin x945/y400 wheel에서도 동일했다. 접근성 좌표 제한을 무작정 늘리거나 가려진 버튼을 강제로 실행하지 않았다. root에 중첩 diagnostic-controls의 max_height만 설정한 높이 예약/parent available-height 상호작용을 조사하도록 보고했다. 제품 결함 확정 또는 최대보고서 GUI 성공을 뜻하지 않는다.

각 full-4~8 result.json은 owned_channels_reaped=true, binary SHA before/after 일치를 기록한다. 사용자 desktop/clipboard를 사용하지 않았다. 최대보고서 copy/save/collision/originaljob 불변/정상 CloseWait는 아직 실행하지 못했다. source fixture 크기 unit과 GUI 성공을 구분한다. full-3에는 terminal result가 없으므로 성공으로 집계하지 않는다.

## 높이 예약 수정 후 실제 GUI (full-9/10)

root가 중첩 diagnostic-controls에 min_scrolled_height(300)를 추가한 CPU SHA `2d6f64a41fb77cae4e83bbc3f873179d9a7531859f430a002f2b1b102ea9fb01`로 재검증했다. full-9에서는 copy 버튼 center가 y369까지 이동해 접근 문제가 해소됐고 Markdown/JSON 복사를 실제 수행했다. 이어진 저장 경로 검색은 harness가 outer 폼만 스크롤해 FAIL로 보존했다. full-10은 outer margin(x945/y400)과 inner controls(x100/y310)를 구분해 visible geometry를 확인한 후 AT-SPI Action.DoAction으로 실행했고 exit0/PASS였다. content clip y535 제한을 확장하거나 숨겨진 action을 강제로 실행하지 않았다.

- Markdown: 1,933,100B, 실제 private Xvfb clipboard 복사 2.952초.
- JSON: 1,841,853B, 복사 1.616초, logs128×4096B 및 attachments8×nodes256을 역직렬화로 확인.
- JSON 새 파일 export와 복사 JSON 내용 일치, 같은 경로 재저장 시 원 export bytes 보존.
- 보고서 format 전환 뒤 원 Download job의 ID/Failed 상태/revision3/Attempt1/options snapshot이 동일했다.
- 소유 창 닫기 후 정상 exit0, 소유 app/DBus 채널 회수, binary SHA before/after 동일. host /proc PID/start identity로 원 app/DBus 부재를 별도 확인했다.
- 보존한 SQLite integrity_check=ok, 원시 JSON/export 실제 크기와 내용 일치를 별도 확인했다.

근거는 evidence/cpu-report-stress/full-10/result.json, large.md, large.json, export.json, jobs.sqlite, PNG에 있다. full-9 실패도 삭제하지 않았다. 정확 2MiB GUI, 최대값 수학적 증명, 다른 OS GUI, 원 MPD 입력 파일의 after-hash, busy export 중 CloseWait는 이 시험의 검증 범위가 아니다. 저장 완료 뒤 정상 종료를 busy I/O 취소 성공으로 표현하지 않는다.

### 발견한 성능 게이트

최대 유효 보고서 작업 뒤 VmHWM524,100KiB(약511.8MiB), VmRSS521,492KiB(약509.3MiB)였다. 기능 동작 PASS가 제안 UI80MiB 예산 PASS를 의미하지 않는다. 전체 ~1.9MB preview TextEdit layout 비용이 의심되며 root에 bounded visible preview/전체 copy-export 분리 개선을 요청했다. 원 보고서/복사/export 전체내용을 축소하여 비용만 낮추는 변경은 요구사항을 충족하지 않는다. 이 시점의 성능 개선은 미검증이었으며, 아래 페이지화 후 실제 GUI로 재검증했다.

## 페이지화 후 비용·전체 내용 회귀

화면 페이지화 CPU SHA `166946017de0b3d0589bf3d02bd864d004d90f3457ff2e314fe5e10a5f865417`의 paged-3/4는 같은 유효 fixture로 exit0/PASS다. 전체 정제 preview String과 복사/export는 유지하고, readonly TextEdit가 읽는 slice만 UTF-8 최대8192B로 제한했다. 새 accessible_preview 모듈이 경계 index를 보관하며 최대2MiB에 유한 페이지 수(최대257)를 갖는다. 실제 JSON225페이지에서 직접 page selector·다음·이전·첫/끝 접근을 native AT-SPI로 검증했다.

| 표본 | HWM | 종료 전 RSS | Markdown/JSON bytes |
|---|---:|---:|---:|
| full-10 전체 TextEdit | 511.8MiB | 509.3MiB | 1,933,100 / 1,841,853 |
| paged-3 | 70.5MiB | 66.8MiB | 동일 |
| paged-4 (이전/다음 포함) | 70.8MiB | 66.8MiB | 동일 |

같은 대형 자료의 HWM이 약86.2% 줄었다. 이는 전체 text galley/layout이 주된 비용이었다는 가설을 지지한다. report 자체를 잘라내거나 copy/export를 일부 page로 바꾸지 않았다. 표본의 HWM은 제안80MiB 안이지만 모든 job/페이지/OS/장시간 peak 예산의 증명이 아니다. paged-4 복사 종단 시간은 Markdown2.279초/JSON1.206초이며 widget 탐색·AT-SPI 호출·대기·clipboard 전송을 포함한다. 제품 frame latency로 해석하지 않는다.

paged-4 first-page.txt8192B/last-page.txt6845B는 실제 Text.CharacterCount와 GetText로 얻었고 전체JSON prefix/suffix와 일치했다. visible preview clip 교차를 확인했으며 selector225/225, disabled 다음 버튼과 실제 preview PNG를 보존했다. 실제 다음 버튼은 두 번째 페이지로, 이전은 첫 페이지로 돌아왔다. 같은 실행 JSON clipboard/export 전체 bytes가 정확히 같고, baseline과 비교하면 생성 report_id/created_unix_seconds만 달랐다. 이 두 정상 생성 metadata만 맞추면 JSON/Markdown 전체내용이 일치한다.

새 파일 저장/같은 경로 collision 보존/원 Failed job revision3·Attempt1·options 불변/정상 exit0/source SHA 유지/소유채널 회수를 재검증했다. SQLite에 저장한 job_json과 최초 Failed 관측이 동일했고 integrity_check=ok, job/attempt 각1개다. host /proc PID/start identity로 소유 app/bus 부재를 확인했다. independent-verification.json에 원시 비교 결과를 보존했다.

paged-1은 마지막 짧은 page에서 GetText end8192를 고정 요청한 시험 오류, paged-2는 SQLite 일시 lock을 관측 함수가 즉시 실패 처리한 시험 오류다. 전용 harness만 실제 CharacterCount/locked 재관측으로 보정하고 원시 실패를 남겼다. 제품 실패 또는 성공으로 바꾸어 집계하지 않는다. 사용자 desktop·clipboard는 사용하지 않았다.

실행: `python3 scripts/cpu_report_stress_probe.py --binary target/release/dlpad-cpu --output docs/validation/evidence/cpu-report-stress/paged-4 --expected-sha 166946017de0b3d0589bf3d02bd864d004d90f3457ff2e314fe5e10a5f865417` (exit0). 전용 Python 구문 검사와 문서 검사를 추가 실행했다. Rust 통합은 root 담당으로 CPU all-target clippy/71개 entry test/release 성공을 보고받았으며 root가 최종 전체 통합 검증한다.

이 기록은 Linux의 최대 유효 fixture와 세 페이지 접근 회귀에 한정한다. exact2MiB 실제 생성, 전체225페이지 GUI순회, other OS GUI, busy export 중 취소/CloseWait, 실제사이트/OS clipboard release 검증을 완료로 주장하지 않는다.
