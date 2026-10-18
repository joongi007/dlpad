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
