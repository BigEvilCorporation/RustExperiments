extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

const pi: f32 = 3.14159265359;
const pi2: f32 = pi/2.0;
const pi180: f32 = pi/180.0;

const screenWidth: u32 = 640;
const screenHeight: u32 = 480;

const gridSize: u32 = 64;

const mapWidth: usize = 16;
const mapHeight: usize = 16;

const fovDegrees: i32 = 60;
const pixelsPerColumn: i32 = (screenWidth as i32) / fovDegrees;

const map: [[u8; mapWidth]; mapHeight] =
[
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,1,1,1,1,1,1,1,1,1,0,0,1],
    [1,0,0,0,1,0,0,0,0,0,0,0,1,0,0,1],
    [1,0,0,0,1,0,0,0,0,0,0,0,1,0,0,1],
    [1,0,0,0,1,0,0,0,0,0,0,0,1,0,0,1],
    [1,0,0,0,1,0,0,0,0,0,0,0,1,0,0,1],
    [1,0,0,0,1,0,0,0,0,0,0,0,1,0,0,1],
    [1,0,0,0,1,0,0,0,0,0,0,0,1,0,0,1],
    [1,0,0,0,1,0,0,0,0,0,0,0,1,0,0,1],
    [1,0,0,0,1,1,1,1,1,1,1,1,1,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],
];

trait Moveable
{
    fn MoveX(&mut self, val: f32);
    fn MoveZ(&mut self, val: f32);
    fn Turn(&mut self, val: f32);
}

struct Entity
{
    x: f32,
    y: f32,
    z: f32,
    r: f32
}

struct Input
{
    up: bool,
    down: bool,
    left: bool,
    right: bool
}

impl Default for Input
{
    fn default() -> Input
    {
        Input { up: false, down: false, left: false, right: false }
    }
}

impl Moveable for Entity
{
    fn MoveX(&mut self, val: f32)
    {
        self.x += (self.r - pi2).sin() * val;
        self.z += (self.r - pi2).cos() * val;
    }

    fn MoveZ(&mut self, val: f32)
    {
        self.x += self.r.sin() * val;
        self.z += self.r.cos() * val;
    }

    fn Turn(&mut self, val: f32)
    {
        self.r += val;
        if self.r >= pi*2.0 { self.r -= pi*2.0; }
        else if self.r < -pi*2.0 { self.r += pi*2.0; }
    }
}

fn Raycast(x: f32, y: f32, ang: f32, grid: f32, boundsX: f32, boundsY: f32) -> (f32, f32, f32)
{
    let mut rayPosX = x;
    let mut rayPosY = y;
    let mut rayDist: f32 = 0.0;

    // Ray angle to dir
    let rayDirX = ang.sin();
    let rayDirY = ang.cos();

    // Offset depending on sign
    let tileOffsetX: i32 = if rayDirX > 0.0 { 1 } else { 0 };
    let tileOffsetY: i32 = if rayDirY > 0.0 { 1 } else { 0 };
    let tileDeltaX: i32 = if rayDirX > 0.0 { 1 } else { -1 };
    let tileDeltaY: i32 = if rayDirY > 0.0 { 1 } else { -1 };

    // Round ray pos down to nearest grid corner
    let mut tileX: i32 = ((x / grid).floor()) as i32 + tileOffsetX;
    let mut tileY: i32 = ((y / grid).floor()) as i32 + tileOffsetY;

    'rayLoop: loop
    {
        if map[tileX as usize][tileY as usize] != 0
            || rayPosX <= 0.0 || rayPosX >= boundsX
            || rayPosY <= 0.0 || rayPosY >= boundsY
        {
            break 'rayLoop;
        }

        let tileXWorldSpace = (tileX as f32) * grid;
        let tileYWorldSpace = (tileY as f32) * grid;

        // Calculate ray delta to next grid lines
        let deltaX = if rayDirX == 0.0 { 0.0 } else { (tileXWorldSpace - rayPosX) / rayDirX };
        let deltaY = if rayDirY == 0.0 { 0.0 } else { (tileYWorldSpace - rayPosY) / rayDirY };

        // The smaller of the two deltas leads us to our first intersection
        let mut deltaT = 0.0;
        
        if deltaX != 0.0 && deltaX < deltaY
        {
            deltaT = deltaX;
            rayDist += deltaX;
            tileX += tileDeltaX;
        }
        else
        {
            deltaT = deltaY;
            rayDist += deltaY;
            tileY += tileDeltaY;
        };

        rayPosX += rayDirX * deltaT;
        rayPosY += rayDirY * deltaT;
    }

    return (rayPosX, rayPosY, rayDist);
}

fn CreateRect(ent: &Entity, w: f32) -> sdl2::rect::Rect
{
    return sdl2::rect::Rect::new((ent.x-(w/2.0)) as i32, (ent.z-(w/2.0)) as i32, w as u32, w as u32);
}

fn CreateLine(ent: &Entity, len: f32) -> (sdl2::rect::Point, sdl2::rect::Point)
{
    return  (sdl2::rect::Point::new(ent.x as i32, ent.z as i32),
             sdl2::rect::Point::new((ent.x + ent.r.sin() * len) as i32, (ent.z + ent.r.cos() * len) as i32));
}

fn DrawGrid(canvas: &mut sdl2::render::WindowCanvas, width: u32, height: u32, cellSize: u32)
{
    for x in (0..width).step_by(cellSize as usize)
    {
        canvas.draw_line(sdl2::rect::Point::new(x as i32, 0),
                         sdl2::rect::Point::new(x as i32, height as i32));
    }

    for y in (0..height).step_by(cellSize as usize)
    {
        canvas.draw_line(sdl2::rect::Point::new(0, y as i32),
                         sdl2::rect::Point::new(width as i32, y as i32));
    }
}

fn main() -> Result<(), String>
{
    let mut camera = Entity { x: 80.0, y: 0.0, z: 80.0, r: 0.0 };
    let mut input = Input::default();

    let sdlContext = sdl2::init()?;
    let sdlVideo = sdlContext.video()?;
    let sdlWindow = sdlVideo
                    .window("Raycaster3D", screenWidth, screenHeight)
                    .position_centered()
                    .opengl()
                    .build()
                    .map_err(|e| e.to_string())?;
    let mut sdlCanvas = sdlWindow
                    .into_canvas()
                    .build()
                    .map_err(|e| e.to_string())?;
    let mut sdlEvents = sdlContext.event_pump()?;

    let mut drawMap: bool = true;

    'mainLoop: loop
    {
        for event in sdlEvents.poll_iter()
        {
            match event
            {
                Event::Quit { .. } => break 'mainLoop,
                Event::KeyDown { keycode: Some(Keycode::W), .. } => input.up = true,
                Event::KeyUp { keycode: Some(Keycode::W), .. } => input.up = false,
                Event::KeyDown { keycode: Some(Keycode::S), .. } => input.down = true,
                Event::KeyUp { keycode: Some(Keycode::S), .. } => input.down = false,
                Event::KeyDown { keycode: Some(Keycode::A), .. } => input.left = true,
                Event::KeyUp { keycode: Some(Keycode::A), .. } => input.left = false,
                Event::KeyDown { keycode: Some(Keycode::D), .. } => input.right = true,
                Event::KeyUp { keycode: Some(Keycode::D), .. } => input.right = false,
                Event::KeyDown { keycode: Some(Keycode::Tab), .. } => drawMap = !drawMap,
                _ => {}
            }
        }

        if input.up
        {
            camera.MoveZ(2.0);
        }
        if input.down
        {
            camera.MoveZ(-2.0);
        }
        if input.left
        {
            camera.Turn(-0.2);
        }
        if input.right
        {
            camera.Turn(0.2);
        }

        sdlCanvas.set_draw_color(Color::RGB(255, 0, 0));
        sdlCanvas.clear();

        if drawMap
        {
            sdlCanvas.set_draw_color(Color::RGB(255, 255, 255));
            DrawGrid(&mut sdlCanvas, screenWidth, screenHeight, gridSize);

            sdlCanvas.set_draw_color(Color::RGB(255, 255, 255));

            let camPosRect = CreateRect(&camera, 4.0);
            let camDirLine = CreateLine(&camera, 8.0);
            
            sdlCanvas.draw_rect(camPosRect);
            sdlCanvas.draw_line(camDirLine.0, camDirLine.1);
        }
        else
        {
            sdlCanvas.set_draw_color(Color::RGB(255, 255, 255));
        }

        let mut columnX: i32 = 0;

        for i in -(fovDegrees/2)..fovDegrees/2
        {
            let rayAng: f32 = camera.r + pi180 * (i as f32);
            let rayHit = Raycast(camera.x, camera.z, rayAng, gridSize as f32, screenWidth as f32, screenHeight as f32);
            
            if drawMap
            {
                sdlCanvas.draw_line(sdl2::rect::Point::new(camera.x as i32, camera.z as i32),
                                sdl2::rect::Point::new(rayHit.0 as i32, rayHit.1 as i32));
            }
            else
            {
                if rayHit.2 > 0.0
                {
                    // Fix fisheye effect
                    let rayDistCorrected = rayHit.2 * (camera.r - rayAng).cos();

                    let mut columnHeight: u32 = (((gridSize * screenHeight) as f32) / rayDistCorrected).floor() as u32;
                    if columnHeight > 0
                    {
                        if columnHeight > screenHeight
                        {
                            columnHeight = screenHeight;
                        }

                        let columnY: i32 = ((screenHeight / 2) - (columnHeight / 2)) as i32;
                        let columnRect = sdl2::rect::Rect::new(columnX, columnY, pixelsPerColumn as u32, columnHeight);
                        sdlCanvas.draw_rect(columnRect);
                    }
                }

                columnX += pixelsPerColumn;
            }
        }

        sdlCanvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
    }

    Ok(())
}
