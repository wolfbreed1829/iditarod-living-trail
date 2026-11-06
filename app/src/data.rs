#[derive(Clone, Copy, PartialEq)]
pub struct Place {
    pub slug: &'static str,
    pub name: &'static str,
    pub region: &'static str,
    pub season: &'static str,
    pub activity: &'static str,
    pub image: &'static str,
    pub alt: &'static str,
    pub intro: &'static str,
    pub story: &'static str,
    pub resource: &'static str,
    pub caption: &'static str,
}
pub const PLACES: [Place; 3] = [
    Place {slug:"seward", name:"Seward & Resurrection Bay", region:"Southern trail", season:"All seasons", activity:"Coast & culture", image:"waterfront", alt:"Mountains across Resurrection Bay from the Seward waterfront", intro:"Begin where the mountains meet the sea.", story:"At the southern end of the historic route, Seward offers an approachable connection to the trail. Start with the waterfront, discover local history, and explore the relationship between a coastal community and the journeys that began here.", resource:"https://www.iditarod100.org/plan-your-trip.html", caption:"Resurrection Bay from Seward · NPS / Max Odland"},
    Place {slug:"chugach", name:"Chugach country", region:"Southcentral Alaska", season:"Summer", activity:"Hiking & exploring", image:"summer", alt:"An aerial view of a glacier and inlet in Chugach National Forest", intro:"Find a new perspective in Southcentral Alaska.", story:"The southern trail passes through a landscape of mountains, forests, and water. Explore official visitor resources for the Chugach region and historic trail connections, including Crow Pass. Choose an outing suited to your experience and check with the relevant land manager before setting out.", resource:"https://dnr.alaska.gov/parks/aspunits/chugach/recops.htm", caption:"Chugach National Forest regional landscape · USDA Forest Service. Not a view of Crow Pass."},
    Place {slug:"winter-trail", name:"The winter trail", region:"Across the trail", season:"Winter", activity:"Winter travel", image:"cycling", alt:"Two people riding fat tire bicycles along the snow-covered Iditarod Trail", intro:"Experience the season that shaped the trail.", story:"Snow and frozen ground connect a landscape that changes dramatically through the year. Skiers, dog teams, snowmachines, and winter cyclists continue a long relationship with winter travel. Remote sections require substantial preparation and local knowledge; begin with the BLM visitor resources.", resource:"https://www.blm.gov/visit/iditarod-trail", caption:"Winter cycling on the Iditarod Trail · Bob Wick / BLM"},
];
#[derive(Clone, Copy, PartialEq)]
pub struct Story {
    pub slug: &'static str,
    pub title: &'static str,
    pub category: &'static str,
    pub image: &'static str,
    pub alt: &'static str,
    pub intro: &'static str,
    pub paragraphs: [&'static str; 3],
    pub source: &'static str,
}
pub const STORIES:[Story;3]=[
    Story{slug:"a-living-connection",title:"A living connection across Alaska",category:"HISTORY & HERITAGE",image:"dawn",alt:"Winter light on mountains and spruce along the Iditarod Trail",intro:"The story of the trail begins with connection: between places, between people, and between generations.",paragraphs:["Long before a named gold-rush route appeared on maps, Alaska Native travel networks connected communities across this landscape. Those connections belong at the beginning of the trail's story, and remain part of its living present.","Later, the movement of mail, supplies, and people linked coastal ports and Interior settlements. The trail carries many histories, shaped by the communities along it and the seasons that make travel possible.","Preserving this heritage means listening as well as looking. The University of Alaska Fairbanks' Project Jukebox brings together recorded voices and personal perspectives. Follow those stories to learn from the people who know the trail."],source:"https://jukebox.uaf.edu/"},
    Story{slug:"the-winter-way",title:"The winter way",category:"LIFE ON THE TRAIL",image:"dogs",alt:"A sled dog team resting on straw in the snow",intro:"For generations, winter has opened a way across Alaska.",paragraphs:["A winter trail is a relationship with a changing landscape. Snow, frozen rivers, and seasonal knowledge shape where and how people travel.","Dog teams are part of that story. So are the people who maintain routes, share knowledge, and connect neighboring communities. Today the trail also welcomes different forms of recreation, from skiing to winter cycling.","The historic trail is a network, with different landowners, conditions, and access requirements. Learning about it is an invitation to understand the landscape before planning a journey through it."],source:"https://www.blm.gov/programs/national-conservation-lands/national-scenic-and-historic-trails/iditarod"},
    Story{slug:"the-next-generation",title:"The next generation of trail stories",category:"LEARNING & STEWARDSHIP",image:"summer",alt:"Glacier, mountains, and forest in the Chugach region",intro:"A place becomes more meaningful when we learn how to care for it.",paragraphs:["A trail can be an outdoor classroom. Observing a landscape, asking about a place name, and listening to a local story can open new ways of understanding where we live.","The Alliance's Iditarod Trail to Every Classroom program connects place-based learning with stewardship and community. Its existing resources offer starting points for educators and curious families.","Explore the Junior Trailblazer materials, discover a passport stamp location, or browse the education program. Start with a question, then follow it somewhere new."],source:"https://www.iditarod100.org/itrec-education-program.html"},
];
