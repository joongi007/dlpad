# S04: Windows 고정 Deno·공식 bundled EJS offline 인수

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R05, R08
- 담당/수정 범위: diagnostic_cache; 신규 scripts/windows_deno_ejs_offline_probe.py, 본 문서 및 fresh evidence
- 선행 작업: Deno2.9.7 고정 bytes, 공식 yt-dlp2026.08.19/bundled EJS exact source·upstream 합성 fixture

## 사용자 결과

공식 bundled solver를 실제 Windows Deno에서 로컬 합성 challenge만 실행하여 node-shim opt-out과의 호환 근거를 얻는다. 계산 sum을 solver로 대신하지 않는다.

## 계약과 범위

공식 source/fixture만 bounded 다운로드, real YouTube/치지직/계정/영상/remote components 없음. 실제 JS argv·권한은 upstream 확인 후 root에게 먼저 전달한다. owned copy/cache/disable-node-shim1·원 private CLI host 사용. Rust/Cargo/shared 환경 수정 없음.

## 인수 조건

공식 release/commit/ZIP/bundledsolver/fixture SHA 연결, actual solver challenge expectedoutput, network deny/importremote0/ownedcache32MiB/nlink1 보존. upstream fixture 없으면 확보 자료와 gap을 명시한다. 실제 EJS 인수 및 공급 채택을 구분한다.

## 검증과 handoff

공식 EJS README/yt-dlp EJS guide primary 조회 완료. 현재 fixture·bundle 조사 중. https://github.com/yt-dlp/ejs 및 https://github.com/yt-dlp/yt-dlp/wiki/EJS .

## 실제 자료와 승인된 좁은 실행

공식 고정 [yt-dlp2026.08.19 ZIPapp](https://github.com/yt-dlp/yt-dlp/releases/download/2026.08.19/yt-dlp) 3,072,469 B/SHA256 `1fa6733c37ea6fb51c99ad8fe785e7b7e5f3246c9b980230329d4fb72ed8d4d6`를 실제 재해시하고 공식 release API asset digest와 비교했다. Windows standalone exe 내부를 검증한 것이 아니라 공식 ZIPapp의 bundled Python EJS 자료를 사용했다. bundled `_version.py`는0.8.0이다. lib.min.js151,561 B/`c55987fe697e5b9ee18830163f7af85327e9bb5c3e674b969d38c8d205eaa577`, core.min.js6,945 B/`18da6ce0758b416e7ae645084f4f8801f9f9d59d6c477c05eaa0ff94ebd8cc00`를 create-only 추출했다.

[EJS0.8.0 정확 revision](https://github.com/yt-dlp/ejs/commit/4fb477f4af56880cfd324c48bd4294a2d2294e50) source archive와 main/solvers/upstream test source bytes/SHA를 수집했다. upstream tests에는 입력/expected가 있지만 player JS는 download.ts로 실제 YouTube에서 별도 받는 구조이며 source players에는 .gitignore만 있다. 따라서 완전한 upstream player fixture를 확보하지 않았고 실제 player JS를 다운로드하지 않았다. self-authored sum을 solver로 대체하지 않았다.

주 담당이 승인한 실행 계약은 **공식 bundled preprocessed API**에 로컬 자체 작성 n reverse/sig slice fixture를 넣고 unknown type/missing function 오류를 확인하는 범위다. 실제 upstream fixture·AST 추출·사이트 challenge 성공과 구분한다. 공식 lib→Object.assign(globalThis,lib)→core→jsc 호출 연결을 사용했다. argv는 Deno run --no-config --no-lock --no-prompt --deny-read --deny-net --deny-env <ownedsolver.mjs>이며 extra grant0/import0/network0, own DENO_DIR/disable-node-shim1이다. combinedJS 및 fixture SHA도 기록했다.

## 실제 검증 결과

`python3 -m py_compile scripts/windows_deno_ejs_offline_probe.py` exit0. `python3 scripts/windows_deno_ejs_offline_probe.py /mnt/d/project/dlpad/docs/validation/evidence/windows-deno-ejs-preprocessed-first --execute` session47064 terminal exit0/PASS, 소유 핸들 종료. compile/native-before/bundled-ejs/native-after 네 original private CLI host 명령은 exit0/private-desktop/input-desktop-unchanged/source_unchanged다. 네 source_after를 당시 현재 root source snapshot과 독립 대조하여 일치했다. 이후 root source 변경에 소급하지 않는다.

실제 bundled core는 n `dlpad-synthetic`→`citehtnys-dapld`, sig `fixture-signature`→`ixture-signature`, unknown request error 및 missing sig error를 정확히 반환했다. native FILE_ID_INFO volume13266031973415618410/fileID27150400000032000000000000000000 및 FILE_STANDARD_INFO nlink1/97,462,048 B가 전후 같다. cache 논리259,008 B≤32MiB, node.exe 부재, 원 Deno/extracted bundle/combinedJS의 전후 SHA를 확인했다. bundled-ejs outer elapsed3.65초는 host/snapshot 비용을 포함하며 단독 solver time·RSS는 미측정이다.

이 근거는 Deno2.9.7 opt-out에서 공식 EJS0.8.0 bundle의 **preprocessed API**가 로컬 함수·오류를 처리한다는 scoped PASS다. Windows standalone bundle 동등성, upstream 전체 player fixture/AST 추출, 실제 사이트, 제품 yt-dlp→guardian typedenv 전달·managednlink 회귀, 공급 trust/notice/source/P0는 별도다. 공식 EJS README의 prebuilt lib는 meriyah ISC/astring MIT를 포함하므로 Unlicense만으로 notice 완비를 주장하지 않는다. 원 preparation evidence와 실제 실행 증거를 둘 다 보존했다.
