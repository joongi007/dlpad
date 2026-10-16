# S04/S05: 실제 production API의 공개 YouTube1~4초 구간 저장

- 상태: proposed / 준비 완료·실제 미실행
- 분류: verification
- 우선순위: P0
- 요구사항: R04, R05, R10
- 담당/수정 범위: search_reveal, B/p0-public-bounded-cut-v2만. Root가 actual 실행·docs/work-items 채택·통합 인수를 담당한다.
- 선행 작업: Root가 exact current whole/도구 원proof 지정, D여유2GiB 확인. 새로운 제품cap schema는 선행 조건이 아니다.

## 사용자 결과

원 YouTube의1~4초 구간을 실제 DownloadRepository/Session으로 저장하고 실제 DB/Attempt/Artifact·미디어 길이/decode로 확인한다. setup/site/budget 실패를 성공으로 바꾸지 않는다.

## 계약과 범위

DECISION.md에 Root의 과도한 hard cap 선행가정 수정을 기록했다. 보호는 실제지원typed range/낮은화질/정확section argv/전체fallback없음·90초협조취소/150초ownedJob·fresh temp/nativeID/SHA·D여유·boundedprogress/redactor다. output32MiB/free1.5GiB는표본soft budget이고 hard quota가 아니다. user계정/쿠키/전체fallback/업로드0. rawCLI를 engine 성공으로 대체하지 않는다. managed환경/GUI/취소·재개/8h실험scope는별도다. Live/S/sharedCargo/lock/schema수정0.

## 인수 조건

Root가 실제빌드와 source/3PE 불변·원Job0·repoDownloaded/attempt1/artifact1·최종20MiB·duration2.75~3.25초·video≤144p·전체artifactdecode exit0를 독립 검토한다. 실패/skip/overshoot/cancel/ownership불명·partial은원상태로보존하며 원cancelACK를 watcher ACK로 대신하지 않는다.

## 검증과 handoff

준비 시Python AST·Rustfmt만0. Cargo/native/영상network실행0. argv.template.json과 HANDOFF.md에Root명령·상한·미검증을기록했다. 새scratchCargo.lock은metadata-v3 actual lock의dependency를유지하고localpackage이름만변경했다. Root가--offline --locked actual compile을확인한다. 제품dependency·schema·worker추가0, fixture만감시thread1/32관측/200ms로bounded이고runtime/크기실측전이다.

Root 사전 검토: 정확한 typed section/quality argv, DownloadRepository/Session 실제 경계, 도구 4개 원 pin과 읽기 전용 file identity, output 감시의 soft 의미, 원 자료 보존을 확인했다. whole28을 명시하여 fresh v2 증거에서 실제 실행한다.

## v2 실제 실행 결과

실제 build-fixture 0/public-cut 2, source 및 원 PE/도구 pin 불변이다. 엔진은 Downloaded revision4·Attempt1·Artifact1/55,277B·도구 Exited0을 기록했으나 fixture의 artifact pin 전 검증에서 FAIL_FIXTURE_VALIDATION으로 끝났다. sampled budget cancel 없음/worker21,284ms이며 full media 검증 성공은 아니다. 원 extended canonical Windows 경로와 입력 경로 비교의 차이를 조사하고 기존 DB/파일을 보존한 별도 읽기 검증을 준비한다. ffprobe·decode는 아직 실행되지 않았다.

## 기존 산출물의 후속 읽기 검증

재다운로드 없이 별도 fresh workroot에서 Windows native 읽기 검사를 실행했다. build0/native0과 원 source/3PE/DB/산출물/4tool 보존을 collector가 확인했다. production open_readonly로 원 Downloaded revision4·Attempt1·Artifact1·stored1~4초를 읽었고 pinned ffprobe는3.008초/256×144 VP9+Opus를, FFmpeg는 전체파일decode exit0을 보고했다. old plain 검사는 통과하지만 old lexical starts_with(output)는 false이며 열린 native root/file HANDLE containment는 성공하여 v2 fixture의 정확한 실패절을 재현했다. 원 v2 실패를 덮지 않고 새 읽기 증거에 구분한다. Root의 추가 독립 pin/원로그 검토가 진행 중이다. 원 실행 당시 artifact SHA/ID는 기록 전 실패했으므로 당시 identity까지 소급 입증하지 않는다. GUI/관리 도구 활성화/취소·재개/다른 사이트·전체 P0 인수는 별개다.


Root 추가 독립 검토도 완료했다. 원34pins·2ops0·source674·3PE·원DB/Artifact/tool 불변, 원 native JSON과 summary 일치, Job revision4/Attempt1/Artifact1·stored구간/도구버전·ffprobe3.008초144p·decode0를 대조했다. 새 evidence/p0-public-cut-readonly-review-v1-20261015/root-review.json의 scoped PASS이며 GUI·관리실행·취소·cleanOS·전체 P0 성공은 아니다.
