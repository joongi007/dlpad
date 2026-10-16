# S00: 공개 영상 URL과 화면별 입력·동작 종단 점검

- 상태: implemented (scoped)
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R05, R06, R07, R08, R11
- 담당/수정 범위: root app/engine 계약·통합·실행·검토; 하위 담당 관측기 /tmp 초안과 읽기 전용 감사만. scripts/windows_public_video_private_probe.cs 및 collector, docs/validation/evidence/와 문서 예정.
- 선행 작업: 입력 caret/시스템 폰트 전체 빌드; 원 공식 설치 도구

## 사용자 결과

사용자가 제공한 치지직15441192·YouTubelxOLdQXE2yw의 공개 URL을 앱에 입력해 준비·미리보기·구간/설정·오류 상태를 관측한다. 화면별 실제 버튼 실행, 입력·Tab·선택과 캡처를 통해 UX/시각 문제를 찾아 수정한다.

## 계약과 범위

사용자 explicit URL 시험 허용. 실제 계정·쿠키·비밀 사용/전송 없음. 공개 영상의 metadata/짧은 미리보기만 시험하며 전체 영상 저장은 하지 않는다. 모든 앱 data/output은 소유 TEMP, private desktop. visible fallback/전역 입력/클립보드 없음. 짧은 영상 시험을 8시간 안정성 인수로 대신하지 않는다.

## 인수 조건

각 URL 준비 성공 또는 정직한 실패 상태와 원 worker 정리·기한을 기록한다. 입력 문자열 readback·도구 경로·영상 제목/길이/decoded 화면을 확인하고 range·옵션·폴더/목록·설정/테마의 상태를 캡처한다. 가짜 extractor로 실제 사이트 성공을 주장하지 않는다. 실패 발견 시 원인/수정/재시험과 남은 결정을 기록한다.

## 검증과 handoff

초기 관측: web 도구는 두 URL을 가져오지 못했으며 이를 앱 extractor 결과로 표현하지 않는다. 이후 아래 실제 Rust 도구 경로 시험을 수행했다.

확인된 기능 결함: 실제YouTube 형식 목록에 video-only DASH/HLS가 있는데 preview selector best[]는 muxed 형식만 요구하여 Requested format is not available이 발생했다. Deno 명시 유무 모두 같은 오류이며 Deno만 바꾸는 것으로 해결되지 않는다. 무음 preview는 bestvideo*[height<=360]/worstvideo*로 낮은 해상도 영상 포함 형식을 선택한다. Deno 선택값도 worker snapshot/argv와 원 파일 pin에 포함한다. 이 변경의 root 담당 파일은 engine/media_preview.rs, app/media_workspace.rs, accessible.rs다. 전체 다운로드의 품질 정책은 바꾸지 않는다. 최초 관측 stale ElementNotAvailable FAIL·Job강제정리는 보존했고 제한된 재조회 후 치지직 준비/재생·정지/4페이지·natural0/Job0/input불변을 관측했다. 실제metadata labels는 빌려온 냥시기·04:36:40.000, first PNG는 방송 준비중 화면이다.

popup-final CPU의 치지직 시험은 준비/시간1–2초readback/오디오품질/고급펼침·접힘/preview취소동작/4페이지/재생·일시정지Invoke/natural0/Job0/input불변을 관측했다. 실제PNG에서 준비중영상·닫힌품질popup·입력중앙정렬을 root가 직접 봤다. YouTube는 준비/60초still/시간·품질·페이지·종료가 성공했으나 pausefalse를 완료로 취급하지 않는다. bounded12pass pause-diagnostic은 traversal오류없이 enabled재생버튼을 기록하고 timeoutPNG는 실제generic미리보기오류를 보여준다. 따라서 observer노드누락으로확정하지않고 실제 연속frame실패를 분리진단한다. 이원FAIL/diagnostic증거는보존한다. 전체영상다운로드·cutexecution·물리pointer/IME·cancelterminalACK는미검증이다.

추가 분리진단: 원공식도구+동일metadata URL을메모리내부에만유지한 C#helper에서single172800B/batch0·2 각각2073600B, 모두exit0/stderr0/naturalJob0이다. production Rust execute ignored public 회귀도batch0·2·4 각각12frame으로1passed/0failed(10.29s)다. 이결과로FFmpeg argv 자체실패가원인이라고쓰지않는다. 앱고유worker/guardian/재생시간경로는아직미확정이다. 독립실행용Rust helper의처음rustc컴파일은E0463으로실패했으며실행하지않았다;원로그보존. 새public회귀는명시환경값과허용URL만받고일반whole에서는ignored, signedURL/원metadata/원stderr를출력하지않는다. catalog 오류안내는data형식/용량한계/도구실행실패를구분한다. 새whole은public-ui-batch-reason후보이며이전951후보와source가다르다.

실제새batch-reason관측에서pause버튼은UIA/PNG에존재하지만Invoke중ElementNotAvailable이고상위busy상태변경에취약했다. 실제playback_button을회귀에사용하니ID62B6→A8EC 변화로RED; 전역UiBuilder.id로고정한GREEN과이전AccessKitID의Click전달검증모두PASS다. 타임라인ui.id기반ID는원래안정적이며초기증인회귀가PASS한것을실패로쓰지않는다. 이수정은Invoke취약성을해결하고,앞서별도PNG에관측된연속미리보기오류의원인까지확정했다고쓰지않는다. 새actualGUI/whole검증은public-ui-playback-final후보다.

playback-final 전체검사는test모듈이impl앞에있다는Clippy items_after_test_module에서FAIL했다. warning허용없이모듈을파일끝으로옮겼으며원실패를보존한다. 준비완료뒤프레임을불러오는중에도주소정보확인중으로표시되던header는MEDIA_FRAME_WAIT로구분했다. 새전체검사는playback-clippy-fixed후보다.

## 최종 통합 검증 (2026-10-08)

현재 source610의 Windows MSVC 전체 검사 `windows-msvc-public-ui-playback-clippy-fixed-20261008`에서 fmt/Clippy/test/release 모두 exit0, 41 harness 합계 **953 passed / 0 failed / 17 ignored**다. ignored 공개망 시험은 별도 명시 실행에서 실제 production engine의 0/2/4초 연속 batch 각각12frame을 검증해1passed/0failed였다. 이전 FAIL·RED와 수정 후 GREEN 로그를 보존했다. 의존성·Cargo.lock·공유 schema 변경 없음.

두 실제 공개 URL의 최종 private GUI `windows-msvc-public-youtube-playback-final-20261008`, `windows-msvc-public-chzzk-playback-final-20261008`에서 준비·짧은 무음 미리보기·재생/일시정지 Invoke·1/2초 시간 입력 readback·오디오 품질 선택·고급 펼침/접힘·preview 취소 동작·4페이지 탐색·자연 종료/Job0를 확인했다. root가 원 PNG를 직접 검토했고 원 collector REVIEW_REQUIRED와 별도로 root-review.json에 scoped 판정을 기록했다. YouTube 길이는01:35:22.000, 치지직은04:36:40.000이므로 짧은 영상만 시험한 것이 아니다. 전체 길이 재생·전체 다운로드·실제 cut·취소 terminal ACK·물리 pointer/IME를 검증한 것은 아니다.

최종 layout63PNG(800/960,Light/Dark/restart,앱150/200%), RawInput19PNG·실제 FFmpeg 합성 색 변화, 실제 Windows 폴더 선택 두 흐름을 재검증했다. 라이브러리 선택은 재실행 후 root/1행3B 복원과 DB integrity를 확인했고 다운로드 폴더 선택은 draft 변경 범위다. 비용은 합성1만 파일 목록 idle 3×60초/18창에서 최대 single-core CPU0.786260%·sampled whole-Job WS72,925,184B(69.55MiB), 각 DB1만행·원합성3만파일 보존이다. 재생/다운로드 비용·quiet host·물리 첫 화면 지연은 측정하지 않았다.

남은 인수: 최종 사용자 디자인 평가, 긴 영상 정밀 제어 가시성,200% 밀도/스크롤 맥락, canonical 경로 표시 정돈, library 미디어 연결/로컬 export, 물리 IME/reader/OS DPI, preview managed lease/cache, 서명·clean OS·정식 설치/updater·전체 법적 공지 및 전체P0. 현재 공개 두 URL의 scoped 성공을 모든 사이트 호환으로 일반화하지 않는다.

현재 담당 파일(공유 계약은 root): `crates/engine/src/media_preview.rs`, `crates/app/src/media_workspace.rs`, `accessible.rs`, `catalog.rs`; 입력 작업 담당 파일은 S00-input-caret-typography.md에 기록했다. native 관측기 `scripts/windows_public_video_private_probe.cs`, `scripts/windows_current_native_ui_collect.py`, `scripts/windows_folder_picker_private_probe.cs`와 관련 검증 증거/설계/통합/사용자 안내를 갱신했다. 하위 담당은 /tmp 읽기 전용 입력·원 로그·source/도구 핀·SQLite·비용 감사이며 root가 직접 통합과 native 실행/PNG 검토를 수행했다.

최종 실행 명령과 결과: `scripts/windows_msvc_workspace_probe.py`의 최종 `windows-msvc-public-ui-playback-clippy-fixed-20261008/result.json`과 원 operation argv/log가 fmt/Clippy/test/release exit0를 기록한다. private UI의 실제 argv는 각 collector-result.json/result.json에 보존했다. `python3 scripts/windows_msvc_notice_graph_probe.py --proof docs/validation/evidence/windows-msvc-public-ui-playback-clippy-fixed-20261008/result.json --configuration docs/validation/evidence/windows-msvc-public-ui-playback-clippy-fixed-20261008/effective-configuration.json --output docs/validation/evidence/windows-msvc-public-ui-final-graph-20261008` exit0: graph3명령과 연결된 whole4명령의 원 자연 종료/Job0, 현재 source610 및 graph258를 root/독립 담당이 검토했다. graph만으로 실제 linked code/CRT/std/법적 공지 완결을 주장하지 않는다.

최종 패키지: `python3 scripts/windows_msvc_candidate_package.py --whole-proof docs/validation/evidence/windows-msvc-public-ui-playback-clippy-fixed-20261008/result.json --graph-proof docs/validation/evidence/windows-msvc-public-ui-final-graph-20261008/result.json --output docs/validation/evidence/windows-msvc-public-ui-final-graph-20261008/dlpad-user-preview.zip --native-installer-exe .local-tools/windows/msvc-supply-20261006/target-msvc/x86_64-pc-windows-msvc/release/dlpad-install.exe --native-installer-sha256 edb7c3daea935e2ea103867e89a1a2df534d7f0d78be476d77ecc0dc40ad5529`(실행 argv는 absolute path) exit0. ZIP17,244,580B/466members/SHA256 `e639daf6bd387bb463082a4d5cd2162cea9949854f1b89be0de787aa8d128745`. root CRC/3exeSHA/PE·고정timestamp/permission·원 notice258/missing0를 검토했고 whole appendix43 및 legal/release false를 유지한다. 앱 CPU9,420,288B·CLI974,848B·installer4,290,560B. 구현/검증은 이번 수직 기능의 scoped 완료이며 전체P0와 최종사용자UI 인수를 완료로 표시하지 않는다.

최종 공통 문서 검사: `python3 scripts/check_docs.py` exit0, PASS866documents·required/local file links, 보존95hash-matched upstream notices. 외부 URL/heading anchors는 checker 범위 밖이다.
