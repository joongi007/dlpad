# S11: v0.1.2 업데이트 지원 버전 공개

- 상태: completed
- 분류: release
- 우선순위: P1
- 요구사항: R08, R10, R11
- 담당/수정 범위: Root; README.md, site/index.html, docs/CICD.md, scripts/package_ci_preview.py, 관련 작업 기록; agent는 별도 로컬 검수 도우미만 담당
- 선행 작업: 9fe2045 업데이트 구현·Windows 로컬 전체 검증, 사용자 main/태그/설치파일 공개 승인

## 사용자 결과

GitHub v0.1.2 설치형·포터블 및 공개 서명 metadata를 제공한다. 기존 v0.1.0은 첫 지원 버전을 한 번 수동 설치한다.

## 계약과 범위

2026-10-18 사용자가 이전 자동 승인 거절 이후 이번 정확한 공개 범위를 승인했다. main과 새태그를 normal atomic push했다. 기존 태그/Release는 수정하지 않는다. 비공개키는 로컬 DPAPI 보관하며 외부 공급하지 않는다. 같은 release run의 원 build입력/notice/capture와 package artifact를 확인한 뒤 공개한다. 개인 Documents·실제영상 없이 fixture profile/가상desktop만사용한다.

## 인수 조건

새 hosted CI 전체 성공, 같은 SHA/run 원artifact·ZIP·manifest·PE·notice 검증, 실제 새설치/재검증·설치된CPU의privateGUI, operationalkey 로컬서명과 공개3파일원SHA 대조, 공개 API/다운로드/site갱신을 확인한다. 자동hosted서명과 Authenticode를 완료로 표현하지 않는다.

## 검증과 handoff

진행 중. source9fe2045c5b8a45ca0d2bf6cfcfd6c1f14a0512df; release run39132836927, main CI39132836883. 문서 job은 성공했다. Release Windows native의 기본 테스트/release 빌드 및 fixture strict lint/build는 성공했지만 첫 success 전환 case가 exit1로 실패해 package/draft는 실행되지 않았다. 원 Job은 natural empty/finalActive0/forcedCleanup=false이며 raw stderr와 receipt SHA를 대조했다. selection generation0이며 generic Other만 남아 최초 handshake와 준비 실패는 아직 구분할 수 없다. hosted 실패의 child-error/단계 관측을 보강한 후 원인을 확인한다. 로컬 7 PASS를 hosted 성공으로 대체하지 않는다. 공개전 v0.1.0은 유지한다. 로컬도우미/원로그는 ignored .local-tools/update-publication-20261018 아래 create-only로 보존한다.

Hosted 진단1: source6b819ea2c9b09152959898bfae7c30191c14c33f, run39135953539 failure. A의 entry/preflight/commit/ready/prepare 및 요청3/worker1 뒤 Disk5 gen1을 관측했다. 원 artifact12139474450 SHA d38ad0b04db5b20d70cfa7eda9b273efa892545ef3dc23b54e17b4fb051f4bab을 다운로드했고 `.local-tools/update-publication-20261018/hosted-diagnostic-v1`에 보존했다. 첫 handshake 실패 가설은 배제하고 staging I/O의 native 오류를 추가 수집한다. 공개 보류 유지.

Hosted 진단2: source9e657047d2cac4789381a41ddb9145e058ec509c/run39136860909의 원artifact12139364478 SHA c71606ddd27cfd1f9f2a73392e8eaf918a7c363abea654f91c115d7c08730d36. 전체 ZIP 기록·서명 해시 확인 이후 phase=publish, raw OS32/Stage OwnedCreated/PublicUnconfirmed/CleanupUnconfirmed. 같은 디렉터리 basename 전환에 한정한 native rename 수정을 검증한다. v0.1.1 태그는 실패한 원 소스로 보존하며 공개하지 않고 수정 후보 버전은0.1.2로 올렸다. 기존0.1.0 공개는 유지한다. Cargo.lock은 workspace3개 version만변경하며 의존성 추가0/추가런타임비용0이다.

0.1.2 후보 검증/태그: native whole workspace 1,838 PASS/0 FAIL/44 ignored + diagnostic2 + signed transition7 PASS, Windows2022 independent run39137703881 성공 및 원 artifact 검증 완료. immutable annotated v0.1.2 태그 object7d15e0a67f3f907f1d87d681da6dd457d39afcc7/source9ed25f1297b59bfc922e0659279a56a227614302 정상push. 전체 릴리스 run39138176097은 실행 중이며 아직 공개하지 않았다.

2026-10-19 공개 직전 인수: v0.1.2 전체 release run39138176097 성공. 같은 실행 packages artifact12141341154(SHA fb16aedac118a25f6b654f21474ff414a3394daffc48d28f6e2f6046983c52f0), 원 build입력·notice·compile lifecycle·fixture7을 독립 검증했다. 설치형ZIP SHA348b0205537cf9aa3fe0d9fbdd84b2a9b1ed550a11b355b7dbee10ad9dbe431f, CPU SHA376e0265cd77e3ab0e6669f741c068301137b6619ba25630957230de461634af. 실제 private Windows 설치/재검증·원payload5 hash·설치CPU Light/Dark·800/960·200%·Dark 재실행 유지와4process 자연exit0/두Job0을 Root가 원로그/PNG로 검수했다. 물리 IME·OS DPI·독립 성능 인수는 포함하지 않는다. 설명 nested-scroll은 [후속 작업](S11-update-notes-scroll-followup-20261019.md)이다. 운영키는 로컬 DPAPI로만 보관했고 공개3파일만 새 draft에 공급해 정확7asset을 확인했다. 현재 공개 전이며 설치기 Launch 연결의 별도 최종 증거를 수집 중이다. 원본은 ignored `.local-tools/update-publication-20261018/`, 큰 artifact는 `/tmp/dlpad-v0.1.2-verified-release-20261019/`에 보존했다.

공개 완료: 2026-10-18T15:49:23Z(한국2026-10-19), Release416523660/v0.1.2를 공개했다. 원 공개 API와 정확7개 browser download를 인증 없이 streaming/hash검증해 모두 일치했다. 원 근거는 `published-release-v2.json`, `public-release-v2.json`, `public-download-review-v2.json`이다. 설치기 앱실행 추가시험 native-v5는 installer→headlessparent→UI의 원Job/직계계보/생성/CPU SHA/privateDesktop/HWND nonce를 대조하고 실제UI닫기 뒤 세process exit0·inner/outerJob0·forcedFalse·원소스불변을 확인했다. Root가 원로그와 installed-ui.png를 직접 검수해 `acceptance-v2.json`에 집계했다. 도우미 v1~v4의 interop/컴파일/창채택/화면관측 실패는 원본 보존하며 제품실패 또는 성공으로 바꾸지 않는다. public signed 전환과 최종 Pages 검증은 별도 진행 중이다.

최종 2026-10-19 인수 완료: 실제 공식 public 공급 A0.1.1→B0.1.2 다운로드/서명/준비/Restart/BReady 및 같은프로필 cold B선택을70.752초시험에서확인했다. 원AUI종료뒤B생성·같은고정부모/Job·정확B SHA/경로·nonce·모든UI/parent자연exit0·두Job0·입력desktop불변을Root가직접대조했다. 별도agent도원JSON/PNG8개/readonlySQLite Dark·library_entries0을확인했다. [통합인수](S11-update-activation-20261018.md)를따른다. A고정부모자체업데이트를뜻하지않는다.

최종 site: release event relay39142209393→main Pages39142216156으로최신0.1.2 downloadindex를갱신했다. 후속 FAQ문서 배포39142717714는무인증API403으로실패했고, [인증교정](S11-pages-authenticated-discovery-20261019.md)후main d70b56f의 Pages39143294764가build/deploy모두성공했다. 공개index.html의새FAQ와release.json의정확0.1.2 ZIP2링크를직접확인했다. localheadless 공개버전6viewport/theme에서링크·preview/코드미서명안내·키보드/hover/overflow0을검증하고Root도모바일PNG를직접열었다. deviceScaleFactor2시험은WindowsOS DPI시험을뜻하지않는다. FAQ focusoutline/본문간격과앱설명nestedscroll은비차단후속이다.

실행검증: `python3 scripts/test_build_site.py`8PASS, 기존headless browser10시나리오PASS +currentrelease6PASS, `python3 scripts/check_docs.py --source-only`1,142문서PASS, `python3 scripts/check_docs.py`1,237문서PASS/원notice133 hash보존. 문서검사만으로runtime증거를검증했다고주장하지않으며원runtime/capture는위독립검수로대조했다. 태그9ed의전체CI/mainCI성공과문서/workflow후속push CI진행상태를구별한다. 기존failed v0.1.1태그·원실패로그·원Root .git/index.lock은보존했다.
