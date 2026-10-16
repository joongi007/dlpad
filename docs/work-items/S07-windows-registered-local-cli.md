# S07: 개발용 등록 준비·재검증 CLI 연결

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R11
- 담당/수정 범위: root — shared exports/통합/문서/native; diagnostic_cache — /tmp dlpad-local-install.rs 초안(최종bin 담당); metrics_recovery — /tmp native actualCLI fixture 초안; soak_recovery_audit — 독립 parser/cap보존 검토
- 선행 작업: fresh 등록source558 scopedDONE 및 RegisteredInspection source560 scopedDONE

## 사용자 결과

기존 prepare/inspect를유지하고 prepare-register로검증된새복사본을개발용등록, inspect-registered로새프로세스readonly재검증한다. 앱자동실행·active설치·삭제/updater를제공하지않는다.

## 계약과 범위

기존9args/strictflagpairs/nativeabsolute/nooverlap/sourceoriginalpins/canceldeadline/JSON4KiB 계약유지. prepare-register flags는prepare와같고 inspect-registered는inspect와같다. unsigneddevonly/fixed schema1, publishertrust·installercomplete·upgrade·deletionauthority·activeinstallationfalse. 등록실패receipt 전체mutation은앞선prepare가있으므로true; 새registration mutation/capcount는따로표현하고 originalpreparedconfirmedcaps와합계도구분한다. source/manifest/target 조상은등록후에도재검증한다. Cargo/lock새dependency없음.

## 인수 조건

4commands 파싱positive/negative/duplicate/timeout/bounds; actualCLI Unicode 새target등록→별도readonly검사5namespace/원recordseal/nativeIDs/SHA; legacy prepare/inspect 3namespace유지; unknownfull/partial/trustfalse계약위반/changedpayload/foreign/existingtarget refusal·원bytes와source/sentinel보존. parser실패변경0, 등록부분실패보수flag. actualWindows전체4검사 및root독립actualrawJob0/privateinput검토. selfdescribingreceipt는정식installer인수와구분한다.

## 검증과 handoff

현재 source560 hashmap의 Windows GNU 전체 fmt/Clippy/test/release, app_install53회귀·parser3회귀가 PASS다. CPU SHA19c49321…/7,917,568B 불변, CLI SHA557c801f…/577,024B(+52,224B), Cargo.lock 불변이다. 추가 source 전체 SHA 검사는64KiB streaming/같은 deadline이며 신규worker/dependency없다.

실행: `python3 scripts/windows_current_workspace_probe.py docs/validation/evidence/windows-app-registered-cli-workspace-20261006-continue --execute --seconds 900`; `/tmp/dlpad-registered-local-cli-fixture.py`의 explicit CPU/CLI SHA를 고정한16명령; 별도 native FileIdInfo observer compile/query; `/tmp/dlpad-registered-cli-control-run.py`의 private windowless cancel/timeout. root는 원로그·source와compiled pins·정확 ErrorKind·mutation flag·canonicaldomainSHA·root/5원nativeIDs·자연Job0/inputdesktop보존을 대조했다. 독립 검토는 별도파일이다.

실제 취소 partial9,371,648B/기한partial4,259,840B에서 source128MiB/notice/foreign 및 copiedprefix 보존, ready marker부재를 확인했다. 이 제어시험은 현재 CLI의 legacy prepare 호환 인수이며 등록단계중간취소·물리diskstall 인수가 아니다. 동일cap 실패수 보존은 engine 회귀와 구분한다.

[전체 검사](../validation/evidence/windows-app-registered-cli-workspace-20261006-continue/root-review.json), [16명령](../validation/evidence/windows-app-registered-cli-real-20261006-continue/root-review.json), [원 ID](../validation/evidence/windows-app-registered-cli-native-identities-20261006-continue/root-review.json), [실제 제어](../validation/evidence/windows-app-registered-cli-control-20261006-continue/root-review.json). 현재 범위만 완료이며 정식 installer/active전환/cold제거/upgrade/서명/MSVC/최소OS/전체P0는 미완료다.
