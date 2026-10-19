# S11: Windows 정식 등록 통합과 배포

- 상태: in-progress
- 분류: feature
- 우선순위: P1
- 요구사항: R08, R10, R11
- 담당/수정 범위: Root — 공유schema/API·installer_main/bridge·Cargo/lock/version·패키징/CI·CONTEXT/ADR/README/배포문서·전체통합검수
- 선행 작업: v0.1.2 공개와 사용자 Windows등록/제거 개발·배포 명시승인

## 사용자 결과

Windows 정식 등록 통합과 배포를 구현해 일반 Windows 앱처럼 설치 목록에서 찾고 실행·제거한다.

## 계약과 범위

Rust native/current-user, 관리자권한·WebView·shell문자열없음. 기존ZIP7/manifest5와v0.1.2 updater읽기호환유지. 앱소유 shell support는bundle외부 별도namespace이며기존미디어·프로필·외부도구·미확인항목자동삭제없음. sharedexports/Cargo/lock/버전은Root만변경한다. agent는자기worktree/명시파일만편집하며최대3개, Root가원결과를독립검증한다.

## 인수 조건

최초설치/기존0.1.2등록/중복설치/제어판제거/실행중거절/삭제후재설치/긴한글경로/privateGUI를검증하고새태그CI·같은runZIP·로컬운영서명·공개7asset/Site를검증해공개. 기존tag불변/키외부전송0.

## 검증과 handoff

현재환경Windows우선/WSL개발, D:338MiB·/tmp586GiB를확인했다. 신규전체cache다운로드대신기존C:ownedtarget/tmp를재사용한다. 원root .git/index.lock은보존하고정상ec77f46 worktree에서분리한다. 실제명령/결과·비용·미검증항목을완료시기록한다.

담당 분리: engine OS등록 agent는독립os-registration worktree의새 os_registration.rs/전용tests만, app agent는독립app-uninstall worktree의새 installer_uninstall.rs/catalog만편집한다. update_transport는ignored `.local-tools/windows-shell-20261019/private-qa`의별도private GUI검수도우미만편집한다. Root는sharedexports/enginewindows-sys3features·0.1.3workspace3versions·installer_main/installer_os/catalog·CONTEXT/ADR·packaging·통합검수를담당한다. 기존ZIP7/manifest5유지로0.1.2 reader호환을유지한다. 새nativebinding은Registry/COM/Shell이며새crate/다운로드외부runtime추가0. 초기후보의shortcut명은install-ID짧은suffix로다중설치충돌을피하며friendly표시정책은후속이다.

환경 조정: D: 여유가147MiB로감소해새native시험의TMP/TEMP/library fixture는기존C:ownedbuild부모아래고유nonce폴더를사용한다. 과거0.1.2 완료된v3/v4 실패fixture의정확소유temp각30MiB는파일별SHA를검증한tar로/tmp에보존한뒤D:복사본만정리한다. 원raw실패로그/receipt/source/PNG·서명키·사용자자료는변경하지않는다. archive위치는ignored storage-archive.json에기록한다.

중간 실제 검증: native-compile-v5에서 strict workspace/all-targets clippy는통과했으나등록positive4개가OS5로실패했다. 진단singlecase에서current_image의속성조회전용HANDLE로PE bytes를읽는오용을확인했고원HANDLE ReOpenFile/read권한·nativeID대조로수정했다. 다음v6은lint통과뒤COM SetPath E_INVALIDARG로positive4개가실패했으며, 단일native-diagnose-v2에서com-set-path를확인했다. 원실패로그와source-unchanged/private-desktop/input-unchanged/Job자연종료결과를보존한다. 앱제거6Windows회귀는native-app-tests-v1에서6PASS/0FAIL, 패키징Python13PASS, source-only문서1146PASS이다. 이는등록/GUI/배포완료를뜻하지않는다.

검수확장: 제거DELETE권한획득중간실패의pending소실및동시2제거창의readonly핀충돌을코드감사로발견했다. 전체원pending선보존·원cap재시도와설치별worker동일threadlease로보완하고실제busy/retry·중복inspection회귀를추가한다. 앱제거메시지는다른제거창이열린경우를일반실패와구분한다.

호환주의:0.1.2의기존앱업데이트는새UI선택만변경하므로Windows등록을소급추가하지않는다. 앱목록/시작메뉴가필요한기존사용자는새설치기를실행해기존bundle을재검증한뒤Windows등록을명시선택한다. 설치목록DisplayVersion은baseline설치버전이며활성UI버전동기화는후속이다.

후속 검증: native-final-v2의 fmt/strict clippy/등록12회귀(직렬화한긴한글.lnk재로드·원파일ID포함)는PASS이다. 전체workspace시험에서는시험worker가공유 DLPAD_LIBRARY_ROOT를주입해기존 UI2회귀의자체임시저장루트전제를덮어쓴것을확인했다. 제품코드변경없이시험worker에서해당override를제거하고각테스트의cfg(test) 임시경로를유지한native-final-v3를실행한다. 실제GUI검수에는별도임시profile/media경로를계속명시한다.

최종통합감사에서Windows등록후verified복구누락과설치기의단발제거worker가shell실패cap를버리는문제를수정했다. 등록성공/실패후fresh bundle검증결과에따라실행/등록재시도버튼을복구하며실제HWND의enabled상태를회귀검증한다. 등록된앱은설치기에서‘제거방법’을안내하고Windows앱목록의retry-capable 전용제거기에서제거한다. legacy미등록bundle의기존명시제거는유지한다.

전체 검증: native-final-v3는 Windows fmt·strict workspace/all-target clippy·workspace tests·release build가모두exit0, source불변/privateDesktop/input불변/Job자연종료0을확인했다. 전체workspace의원stdout집계는1856PASS/0FAIL/44ignored이고별도등록12회귀도PASS다. 이후cold admission의readonly 등록상태표시만보완해native-final-ui-v1에서설치기회귀/strictlint/release를다시확인한다. 등록된앱의제거버튼은cold확인뒤에도‘제거방법’이며Windows설정의전용제거기를안내한다.

최종 설치기 보완 검증: native-final-ui-v1은 Windows fmt·strict workspace/all-target clippy·설치기53PASS/0FAIL/4ignored·release build 모두exit0이다. 원source불변/privateDesktop/input불변/Job자연종료0을확인했다. 다음은정상worktree의명시19파일을커밋해새0.1.3태그CI로검증하고,같은run원ZIP의가상설치·등록·취소·제거·동일경로재설치를검수한다. 공개전까지작업상태는in-progress다.

## v0.1.3 배포 검수 진행

소스 `0828c42f015b7cd0be78f0829178cbcb8c1da357`을 main에 push하고 새 annotated `v0.1.3` 태그를 생성했다. 태그 이동은 없었다. [일반 CI](https://github.com/joongi007/dlpad/actions/runs/39152374923)와 [릴리스 CI](https://github.com/joongi007/dlpad/actions/runs/39152377040)는 모두 성공했다. 같은 run의 원본 패키지·notice·binding·7개 updater 회귀를 독립 대조했으며 설치형 ZIP SHA256은 `6ca204986f9785bf4dc364dbe66f96d6baa26bbc4a9ed52741e438dcde486ccf`다. 로컬 DPAPI 키로 공개 메타데이터만 서명하여 draft에 총7asset을 준비했다. 개인 키 전송은 없다.

실제 CI 설치기의 private desktop 설치·Windows 등록·앱 실행과 부모/UI 자연 종료0까지 확인하고 원본 PNG를 직접 검토했다. v12/v13 observer는 DOS 경로와 extended 경로의 문자열 동등성 가정을 잘못 적용하여 제거 전에 중단했다. 같은 원 native identity를 검증하도록 observer만 수정 중이며 제품/태그는 그대로다. 제거·취소·동일 경로 재설치 인수와 공개 다운로드 검증 전에는 배포 완료로 보고하지 않는다. 실패한 시험의 원 로그와 소유 설치는 보존했다.

배포 차단: v15 실제 제거에서 본체 제거 후 shell 정리가 실패하여 v0.1.3 draft는 공개하지 않는다. 태그와 실패 증거를 보존한다. 제거 자식의 support 작업 폴더 상속을 native 대조로 재현했고 v0.1.4 최소 수정과 같은 trigger 회귀를 [별도 P0 작업](S07-uninstaller-inherited-cwd-20261019.md)으로 추적한다. v0.1.3 공개7asset/성공제거는 완료되지 않았다.

추가 실제 인수 담당: updater_audit agent는 ignored legacy-012-qa/ 전용 helper만 준비한다. 범위는 기존 공개0.1.2 원 CI 설치를 보존한 새1.4 설치기의 readonly 확인→명시 Windows등록→기존앱실행이며, 새CI1.4원비트검증을 선행한다. 원 source/helper·다른 QA 파일 동시수정은 금지한다. exact원기존CPU/hash/manifest/journal/profile/media/tools 보존 및 등록버튼의 실제 상태/결과가 인수조건이고, 준비 완료를 실제 인수 완료로 쓰지 않는다.

## v0.1.4 실제 완료 근거

[공개 v0.1.4](https://github.com/joongi007/dlpad/releases/tag/v0.1.4)의 소스는 `e8d1007a792c17319e5498cfc6182d174745a4ab`, [태그 CI](https://github.com/joongi007/dlpad/actions/runs/39157614958)와 [main CI](https://github.com/joongi007/dlpad/actions/runs/39157611977)는 모두 성공이다. `python3 .local-tools/windows-shell-20261019/run-native-cwd-fix.py`에서 fmt·strict workspace/all-target clippy·등록12회귀·workspace1,856 PASS/0 FAIL/44 ignored·release build가 모두0이다. hosted 원 stdout도같은1,856/0/44이며 원7개 updater 회귀의 자연 Job0/로그 hash도대조했다.

원 CI ZIP으로 private 실제 설치→등록→앱실행→취소→제거→같은 위치 재설치→재제거를 확인했다. 다섯 단계 원 parent/UI/Job은자연0이며 source/privateDesktop/inputDesktop불변을확인했다. 두 외부 journal의 원5기록씩 SHA를최종제거후다시읽어대조했다. Root가 원 로그와 주요 PNG4개를직접검수했다. 기존 공개0.1.2에 새 설치기로 명시 등록해 원bundle/journal nativeID/bytes 보존, 기존CPU실행·자연Job0, 실제화면3개도확인했다. 합성 profile/media/tools만사용했고 실제사용자 Documents/영상은사용하지않았다.

설치 ZIP SHA256 `7c21482631e666486fe0a190d358e3413f158855441b471aee416f5969502576`, installer SHA256 `bbed56461a9cda0d12928077e491fb81c2805170462bee029fa6de6580940273`이다. 검수한 같은원본을 공개했고7개asset을인증없이내려받아각bytes/SHA를대조했다. 로컬DPAPI개인키는전송하지않고compiled공개키로검증한공개metadata/signature만공급했다. Root원인수는 ignored publication-v0.1.4/acceptance.json 및 legacy-acceptance.json, 공개전달검사는 public-download-review.json에기록했다.

제한: Windows API/COM/private HWND 검수이며 물리설정/제어판·새제거기200%·스크린리더·UNC·다른OS검증은아니다. cold부분제거복구·baseline버전동기화·임시제거복사본자동정리는후속이다. 실제공개1.4앱GUI업데이트전환을별도실행했다고주장하지않으며동일프로토콜signed합성7회귀/공개metadata원bytes검증과구분한다. 기존0.1.2사용자의OS등록은새설치기에서명시해야한다.

현재 배포·원파일 공개검증은 완료했고 소개 페이지/최종 문서 main 반영과 시험 잔여 정리를 마친 뒤 작업 상태를 완료로 갱신한다. 원v0.1.3태그·draft7asset은보존하고검수실패표시를추가했다. draft API가생략한tag_name을임시untagged표기로바꾸는동작을확인해원tag연계를복원했으며Git태그object는불변이다. v0.1.4공개는tag_name/source를명시했고공개API로다시확인했다.

최종 보완(2026-10-19): `python3 scripts/check_docs.py` 1242문서/원notice133hash일치, `python3 scripts/test_build_site.py` 8PASS, 기존headless `scripts/test_site_browser.cjs` 10scenario PASS를확인했다. 사용자desktop표시/전역입력0이며 시험loopback서버와browser는정상종료했다. 문서·사이트는v0.1.4로갱신했고main반영/공개Pages재대조를이어간다.

승인된시험잔여4개와legacy검수설치는정확원소유범위에서제거했고각원Job0·원key/lnk/support/bundle부재및profile/media/tools/journal/source보존을Root가원receipt/정확leaf읽기로대조했다. v12~14의neutralCWD제거는과거실패제품성공으로재분류하지않는다. 본체가이미없는v15는별도원cap도구로정리했으며제품cold부분복구기능을추가했다고주장하지않는다. tool의registrylink/sharedelete/원pin백업및SID문자열오버로드문제는삭제전검토/실패로발견해수정했고원source/log를보존했다. 보존증거/임시실행복사본은삭제하지않았다. Root종합은ignored publication-v0.1.4/cleanup-review.json이다.
