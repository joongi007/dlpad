# S06: Windows CPU에 도달하는 누락 공지문 우선 확인

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R05, R11
- 담당/수정 범위: root .github/workflows/ci.yml·scripts/windows_dependency_notice_scope.py·test_windows_dependency_notice_scope.py·본문·증거·mapping 채택/통합; diagnostic_cache /tmp 읽기감사와mapping초안만
- 선행 작업: wholelock 실제누락43·otherOS3 원공지문 별도미채택감사

## 사용자 결과

Windows 최초패키지에 필요한 원공지문 공백부터 줄인다.

## 계약과 범위

원lock/cache archive/VCS/grant와현재Windows도달범위를구분한다. 추측공지문·archive코드/객체파일을license로채택하지않는다. 소스공지문증거와실제link/재배포권리를동일하게판정하지않는다. 현재developmentZIP은이전snapshot이며신규제품release_ready를주장하지않는다.

## 인수 조건

각후보정확version/checksum/VCS·원grant·Windows실제도달증거와미검증범위, bounded원archive판독·tamper/불일치거절. 공유mapping변경은root검토후만게시한다.

## 검증과 handoff

읽기감사를병렬실행한다. nativeCargo/cache수정과유료/외부게시없음. FFmpeg외부바이너리source/공지문은별도공급게이트이다.

### 現 dependency 범위 분류

기존WindowsCPU normal/build tree와원ZIP43누락의교집합은0으로읽기감사했다. 그러나당시tree sourcepin이없어현재feature도달증거로승격하지않는다. root는 새원native tree receipt·고정argv·lock/source SHA·자연Job/private/input·boundedlog를검증하는분류기를게시했다. workspace package이름이hostbanner로오인되는초안결함은 root검토로발견/교정후회귀를추가했다. wholelock누락은필터로삭제하지않고linked/embedded/legal/release완료false를유지한다.

### 실제 현재 scope 검증

고정nativeWindows Cargo tree receipt는source전후동일/currentSHA와lock을확인했고현재WindowsCPU normal/build259패키지와원missing43교집합은0이다. 초기validator는missing source에는records가없다는inventory계약을잘못가정해ValueError를반환했다. 원native tree를재실행하지않고정확lockkeys와records∪missing의전체대조로교정했으며없는records/미계상lock에대한회귀를추가했다. 현재12개Python 실패/정상시험PASS, CI정의에동일명령을추가했으나remote CI실행은없다. wholelock43·embedded/link/license/releasefalse를그대로보존한다. [원scope](../validation/evidence/windows-current-dependency-notice-scope-20261005-continue/result.json).

### Scoped 독립 인수 완료

새tree259·missing-in-tree0/wholelock43·source현재일치를독립재검산했고 차단결함은없었다. 원12개Python회귀도독립실행PASS다. [독립검토](../validation/evidence/windows-current-dependency-notice-scope-20261005-continue/independent-review.json). 이작업은현재scope분류완료이고 라이선스완비·공급/패키지/embedded/link/릴리스게이트의완료가아니다. 새mapping채택0, 원wholelock누락유지, Git/remoteCI실행없음.
