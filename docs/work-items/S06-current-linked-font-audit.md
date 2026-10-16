# S06: 최신 Windows 내장 폰트 payload와 공지문 감사

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R11
- 담당/수정 범위: diagnostic_cache /tmp asset/source/notice읽기검토·초안; root 검증·프로젝트게시·본 문서
- 선행 작업: 최신CPU전체4명령/개발ZIP2개/원noticeinventory

## 사용자 결과

Windows 후보에 실제 포함된 내장 폰트 원바이트와 원 공지문 연결을 확인해 공급 인수 공백을 줄인다.

## 계약과 범위

원CPU3f9·epaint_default_fonts0.33.3 cache/src/font/grant·현재ZIP공지문을읽기대조한다. 실제asset수는source확인으로결정하며추측하지않는다. PE mapping/fullasset inclusion/원SHA는법적완비·실제렌더사용·reachability의증거와구별한다. 사용자외부폰트/credentials/cache변경0. root만후속패키지/스크립트인터페이스를조정한다.

## 인수 조건

원asset목록과역할·fullbytes/SHA·boundedPE원sectionoffset·같은원grant/noticeZIPbyte대조 및부족문서화. 원binary/source불변·negative변조/경계거절은실제검증도구도입시필요하다. embeddedlegal/최종스택/릴리스완료를추정하지않는다.

## 검증과 handoff

읽기감사시작. 폰트presence선언을exactlinkedpayload검증완료로표시하지않는다.

root가scripts/windows_font_payload_probe.py·windows_font_payload_probe_tests.py를게시했다. child초안의PEheader overlap/optional size상한·exactsource ambiguity·ZIP원CPU/manifest교차대조·ZIP/verifierpostSHA를보강했다. 원crateChecksum·원선택member·4TTF/4notice·sourcecode원SHA와CPU.rdata fullasset exactlyonce를검증하며법적완비/use/stack/release false를유지한다. 테스트는tempownedcopies/syntheticPE만수정한다. 실제검증은후속이다.

실제 CLI를 최신defaultCPU sourceproof/원ZIP에실행하여 PASS. 원cratechecksum5c4fbe../원9selectedmembers·원4TTF wholebytes가CPU.rdata에각1회 포함됨과4공지ZIP원문일치를반복검증했다. actual result.json·negative-result.json(합성PE/소유copy13실패거절)과독립asset대조를windows-current-font-payload-20261005-continue에보존했다. root가4assetSHA/offset/CPU/원ZIP을독립자료와대조했다. runtimeuse/legalcoverage/stack/release는false이며현재ZIPmanifest의과거미검증flag를소급수정하지않는다. 게시helper의후속독립검토는진행중이다.

독립검토에서ZIPentry count검사가central-directory로드후라는한계를확인했다. root가원ZIPbuffer32MiB내EOCD/전체centralheader를사전순회해최대4096개·정확count/range를검사한뒤ZipInfo를할당하도록보강했다. declared4097및forgedEOCDcount1/실제4097 두소유fixture에서ZipFile호출전거절을mock으로확인하는회귀를추가했다. 원firstactual13회귀증거는그snapshot으로보존하고새actual결과에서재검증한다. Python전체RSS32MiB를주장하지않는다.

보강후actual CLI도fresh windows-current-font-payload-bounded-20261005-continue에서PASS. root는현재verifierSHA/CPU/원ZIP/4fullassetoffset과독립원자료를다시대조했고15실패회귀를재실행·저장했다. EOCD과거13검증은별도이력으로보존했다. 정확storedpayload+대응공지문연결의좁은감사범위done이며전체font/legal/SDK/embeddednotice범위완료가아니다.
