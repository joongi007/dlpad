# S06: Windows 후보 라이선스 누락 감사

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R11
- 담당/수정 범위: root는 이 문서·통합·추가자료 채택, soak_recovery_audit는 /tmp/windows_license_gap_audit.md 읽기 감사 초안만 담당한다. product/Cargo/schema/lock 변경은 root만 담당한다.
- 선행 작업: Windows 개발후보 archive의 actual inventory와 독립검토 PASS. 공개릴리스는 미완료다.

## 범위와 인수

lock superset inventory의 누락84건을 workspace/vendor/git/cache missing으로 실제 분류하고 현재CPU feature의 runtime/embedded 관련 자료를 구분한다. 실제 license 표현과 source notice를 읽으며 이름만으로법적승인/linkedclosure완결을 주장하지 않는다. 외부tool·폰트·자료의근거없는bundling이나삭제는없다. root가본자료·경로·후속검증범위를독립대조한다. 소스현재feature의정확closure는 actualtool evidence가없는한추정으로표시한다. 공개게시/유료/서명/계정인증은없다.

실제84누락은 top notice미발견cache package79개와 workspace3/vendor2개다. lock527package superset이므로83/84개를모두실제Windowsbinary license결함으로단정하지않는다. 실제patched vendor/winit/LICENSE는존재하고 CPU의epaint_default_fonts0.33.3 source는4TTF include_bytes를선언한다. root는고정캐시font source/notice경로와앱FontDefinitions::default를대조했다. 시스템한글fallback은defaultfont추가구성을사용하므로기본내장폰트부재를의미하지않는다.

root는packaging도구의not_bundled를external_font_files로정정하고 embedded폰트4source와4notice를boundedinventory에연결했다. completecoverage=false/exactlinkedpayload=false를유지하고note/source누락/선언불일치는failclosed한다. 이후actualZIP재검증은현재product/defaultCPU전체검증근거에맞춰진행한다. source4TTF합계1,407,752B는executable추가용량측정이아니며기존runtime의내용이다. exactlinkedclosure/entireeguiMIT-Apachecopyright/Noto-Ubuntu attribution/workspacelicense/실제notice추가감사는남는다.

2026-10-05 root는 동일 commit3997291f318a8bc06fa522f5579836fb3feb94df의 원 iced0.14.0/LICENSE 1073B/SHAfc9086ba4eba4b77e4c603906362742bc3715ffa2c7810c6074fe7780d3a0398를 exact cached grant로 7component에 연결했다. 기존15mapping을 유지하고22mapping의 fullmanifest와 실제7targets를root가독립확인했다. 담당은soak_recovery_audit의 /tmp utility/manifest/notice초안과 root의scripts/windows_candidate_package.py 및 docs/licenses/upstream/*통합이다. 동일파일 동시수정 없다. 이 결과는 license본문보완이며 전체legal/linkedSBOM완비false를유지한다. 새ZIP검증은source변경전현재proof로별도실행한다.

동일입력 새ZIP2개 독립검증PASS: SHA d11742a0538faacb35c26a6aa6761de10ce7750da872d0e4338674feb52761eb/12,545,439B/838항목, license누락61건. 원CPU snapshot의개발후보이며 이후firstpaint구현의현재binary를포함하지않는다. [원검증](../validation/evidence/windows-candidate-package-iced-notices-20261005-resume/independent-archive-review.json). 이후 donor 경로/타입/pin과파일크기를hash전검증하도록강화했다. 전체배포license/SBOM완비false유지.

### 고정 clipboard-win notice 추가

root가 soak_recovery_audit의 /tmp 초안을 검토해 고정 clipboard-win5.4.1 commit3b27cf2bfd1adcfa6e0264eb51c1025ddaf0f342의 BSL LICENSE1338B/SHA c9bff75738922193e67fa726fa225535870d2aa1059f91452c411736284ad566를 notice manifest와 package utility에 연결했다. 공식 정확 URL·원cache VCS/name/version/license/path 검증, leaf/LICENSE의 정확 grant allowlist를 유지한다. 비용은 notice1338B 및 metadata이며 runtime/dependency/Cargo 변경은 없다. 원 양성1/거절5와 Python compile은 초안 결과이며 root fullmanifest/최신기본빌드 2ZIP 검증은 아직 진행 전이다. 법적 완전성/전체SBOM/릴리스완료를 주장하지 않는다.

root가constify0.0.1/profiling1.0.18 정확cacheVCS/repo/path와고정공식rootMIT/APACHE4notice 초안을검토해추가했다. 추가payload23120B,source12/package25매핑,compiletimeconstify를실제binary포함이라고주장하지않는다. runtime/Cargo변경없으며전체linkedSBOM/법적완비false유지. 최신default2ZIP검증전gap감소완료주장없음.

최신default 두개ZIP생성은성공했으나독립reviewer가선택적capability +필수fourproof를불필요하게거절했다(exit1). root담당 scripts/windows_candidate_package_review.py에정확four또는정확four+성공capability만허용하고선택적record의실제argv/source/exit/private/input도검증한다. unknown operation/실패capability는여전히거절한다. 원실패를package증거에보존하고실제두ZIP재검토와실패변형을확인한다.

서로다른iced_tiny_skia0.14.1/widget0.14.2/winit0.14.1 원revision3개의공식MIT1073B notice를exact원VCS/name/version/license/path검증후root통합했다. 추가3219B+metadata,원399...cachebinding재사용없음,동일SHA라는이유로revision근거를생략하지않는다. 새source15/package28이며runtime/Cargo변경없다. 최신2ZIP원검증전gap감소claim없음.
